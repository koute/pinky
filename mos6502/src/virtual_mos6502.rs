use self::Register8::*;
use self::Direction::*;

use core::fmt;
use core::ops::Deref;

use emumisc::WrappingExtra;

pub type Address = u16;

#[derive(Copy, Clone, PartialEq)]
pub enum EmulationStatus {
    Normal,
    InfiniteLoop( Address )
}

#[derive(Copy, Clone, PartialEq)]
pub enum EmulationError {
    InvalidInstruction( Address, u8 )
}

pub trait Context: Sized {
    fn state_mut( &mut self ) -> &mut State;
    fn state( &self ) -> &State;

    fn peek( &mut self, address: Address ) -> u8;
    fn poke( &mut self, address: Address, value: u8 );
    fn bcd_mode_supported() -> bool;

    #[allow(unused_variables)]
    fn trace( &mut self, pc: u16, opcode: u8 ) {}
}

pub trait Interface: Sized + Context {
    #[inline]
    fn reset( &mut self ) {
        Private::reset( self )
    }

    #[inline]
    fn set_nmi_latch( &mut self, state: bool ) {
        Private::set_nmi_latch( self, state )
    }

    #[inline]
    fn set_irq_line( &mut self, state: bool ) {
        Private::set_irq_line( self, state )
    }

    #[must_use]
    #[inline]
    fn irq_line( &self ) -> bool {
        Private::irq_line( self )
    }

    #[inline]
    fn execute( &mut self ) -> Result< EmulationStatus, EmulationError > {
        Private::execute( self )
    }

    #[must_use]
    #[inline]
    fn get_register( &self, register: Register8 ) -> u8 {
        Private::get_register( self, register )
    }

    #[must_use]
    #[inline]
    fn pc( &self ) -> u16 {
        self.state().pc
    }

    #[must_use]
    #[inline]
    fn set_pc( &mut self, value: u16 ) {
        self.state_mut().pc = value;
    }

    #[must_use]
    #[inline]
    fn fetch( &mut self, address: Address ) -> u8 {
        Private::fetch( self, address )
    }

    #[inline]
    fn dummy_fetch( &mut self, address: Address ) {
        Private::dummy_fetch( self, address )
    }

    #[inline]
    fn write( &mut self, address: Address, value: u8 ) {
        Private::write( self, address, value )
    }
}

impl< T: Context > Interface for T {}
impl< T: Context > Private for T {}

pub struct State {
    /* Registers. */
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    status: u8,

    /* Interrupts. */
    irq_line_state: bool,         /* IRQ is being detected by a level detector. */
    nmi_latch_state: bool,        /* NMI was detected by an edge detector. */
    nmi_pending: bool,

    /* Various. */
    decoded: DecodedOpcode
}

impl State {
    pub const fn new() -> State {
        State {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            status: StatusFlag::Unused.bits() | StatusFlag::IRQDisable.bits(),
            irq_line_state: false,
            nmi_latch_state: false,
            nmi_pending: false,

            decoded: DecodedOpcode {
                pc: 0,
                addr: 0,
                opcode: 0,
                operand: 0,
                opcode_attr: OpcodeAttributes::empty()
            }
        }
    }
}

bitflags!(
    #[derive(Copy, Clone)]
    pub struct StatusFlag: u8 {
        const Carry         = 1 << 0;   /* This holds the carry out of the most significant bit in any arithmetic operation.
                                           After subtraction this flag is cleared if a borrow is required, and set if no borrow is required.
                                           Also used in shift and rotate logical operations. */
        const Zero          = 1 << 1;   /* Set to 1 when any arithmetic or logical operation produces a zero result, and is set to 0 if the result is non-zero. */
        const IRQDisable    = 1 << 2;   /* If set interrupts are disabled. */
        const BCDMode       = 1 << 3;
        const SoftIRQ       = 1 << 4;   /* Is set to 1 all the time, except after an NMI. (Can be accessed only by pushing the status register to stack.) */
        const Unused        = 1 << 5;   /* Is set to 1 all the time. (Can be accessed only by pushing the status register to stack.) */
        const Overflow      = 1 << 6;   /* Set when an arithmetic operation produces a result too large to be represented in a byte. */
        const Sign          = 1 << 7;   /* Set if the result of an operation is negative, cleared if positive. */
    }
);

const NMI_VECTOR_ADDRESS: Address = 0xFFFA;
const RESET_VECTOR_ADDRESS: Address = 0xFFFC;
const IRQ_VECTOR_ADDRESS: Address = 0xFFFE;

impl fmt::Display for StatusFlag {
    #[allow(unused_assignments)] // Since Rust generates an invalid warning here: "value assigned to `already_written` is never read"
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        let mut already_written = false;
        macro_rules! print {
            ($flag:ident) => {
                if self.contains( StatusFlag::$flag ) {
                    if already_written {
                        write!( fmt, "|" )?;
                    }
                    write!( fmt, stringify!( $flag ) )?;
                    already_written = true;
                }
            }
        }
        print!( Carry );
        print!( Zero );
        print!( IRQDisable );
        print!( BCDMode );
        print!( SoftIRQ );
        print!( Unused );
        print!( Overflow );
        print!( Sign );

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Register8 {
    A,
    X,
    Y,
    SP,
    STATUS
}

impl fmt::Debug for EmulationStatus {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        use self::EmulationStatus::*;
        match *self {
            Normal => write!( fmt, "Normal" ),
            InfiniteLoop( addr ) => write!( fmt, "InfiniteLoop[0x{:04X}]", addr )
        }
    }
}

impl fmt::Debug for EmulationError {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        use self::EmulationError::*;
        match *self {
            InvalidInstruction( address, opcode ) => write!( fmt, "InvalidInstruction[0x{:02X} at 0x{:04X}]", opcode, address )
        }
    }
}

impl fmt::Display for EmulationError {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        use self::EmulationError::*;
        match *self {
            InvalidInstruction( address, opcode ) => write!( fmt, "invalid instruction 0x{:02X} encountered at address 0x{:04X}", opcode, address )
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EmulationError {
    fn description( &self ) -> &str {
        use self::EmulationError::*;
        match *self {
            InvalidInstruction( _, _ ) => "invalid instruction encountered"
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
enum Direction {
    Left, Right
}

#[derive(Copy, Clone, PartialEq)]
pub enum Location {
    Imm8( u8 ),
    Imm16( u16 ),
    AbsZP( u8 ),
    Abs( u16 ),
    IndexedZP( Register8, u8 ),
    Indexed( Register8, u16 ),
    IndexedIndirect( u8 ),
    IndirectIndexed( u8 ),
    Reg( Register8 )
}

impl fmt::Debug for Location {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        use self::Location::*;
        match *self {
            Imm8( value ) => write!( fmt, "0x{:02X}", value ),
            Imm16( value ) => write!( fmt, "0x{:04X}", value ),
            AbsZP( addr ) => write!( fmt, "*0x{:02X}", addr ),
            Abs( addr ) => write!( fmt, "*0x{:04X}", addr ),
            IndexedZP( reg, base ) => write!( fmt, "*(0x{:02X}+{:?})", base, reg ),
            Indexed( reg, base ) => write!( fmt, "*(0x{:04X}+{:?})", base, reg ),
            IndexedIndirect( base ) => write!( fmt, "**(0x{:02X}+X)", base ),
            IndirectIndexed( base ) => write!( fmt, "*(*0x{:02X}+Y)", base ),
            Reg( reg ) => write!( fmt, "{:?}", reg )
        }
    }
}

fn show_branch( fmt: &mut fmt::Formatter, flag: StatusFlag, cond: bool, offset_u8: u8 ) -> fmt::Result {
    let offset = offset_u8 as i8;
    if offset == -2 {
        write!( fmt, "HANG" )?;
    }
    else {
        write!( fmt, "JMP {}{}", if offset.wrapping_add(2) > 0 {"+"} else {""}, offset.wrapping_add(2) )?;
    }
    write!( fmt, " IF {}{:}", if cond {""} else {"!"}, flag )
}

impl fmt::Debug for Opcode {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        use self::Opcode::*;
        match *self {
            ADC( arg )  => write!( fmt, "ADC {:?}", arg ),
            AND( arg )  => write!( fmt, "AND {:?}", arg ),
            ASL( arg )  => write!( fmt, "ASL {:?}", arg ),
            BCS( arg )  => show_branch( fmt, StatusFlag::Carry, true, arg ),
            BCC( arg )  => show_branch( fmt, StatusFlag::Carry, false, arg ),
            BEQ( arg )  => show_branch( fmt, StatusFlag::Zero, true, arg ),
            BNE( arg )  => show_branch( fmt, StatusFlag::Zero, false, arg ),
            BMI( arg )  => show_branch( fmt, StatusFlag::Sign, true, arg ),
            BPL( arg )  => show_branch( fmt, StatusFlag::Sign, false, arg ),
            BVC( arg )  => show_branch( fmt, StatusFlag::Overflow, false, arg ),
            BVS( arg )  => show_branch( fmt, StatusFlag::Overflow, true, arg ),
            BIT( arg )  => write!( fmt, "BIT {:?}", arg ),
            BRK         => write!( fmt, "BRK" ),
            CLC         => write!( fmt, "CLR Carry" ),
            CLD         => write!( fmt, "CLR BCDMode" ),
            CLI         => write!( fmt, "CLR IRQDisable" ),
            CLV         => write!( fmt, "CLR Overflow" ),
            CMP( arg )  => write!( fmt, "CMP {:?}", arg ),
            CPX( arg )  => write!( fmt, "CPX {:?}", arg ),
            CPY( arg )  => write!( fmt, "CPY {:?}", arg ),
            DEC( arg )  => write!( fmt, "DEC {:?}", arg ),
            DEX         => write!( fmt, "X--" ),
            DEY         => write!( fmt, "Y--" ),
            EOR( arg )  => write!( fmt, "EOR {:?}", arg ),
            INC( arg )  => write!( fmt, "INC {:?}", arg ),
            INX         => write!( fmt, "X++" ),
            INY         => write!( fmt, "Y++" ),
            JMP( arg )  => write!( fmt, "JMP {:?}", arg ),
            JSR( arg )  => write!( fmt, "JSR 0x{:04X}", arg ),
            LDA( arg )  => write!( fmt, "A <- {:?}", arg ),
            LDX( arg )  => write!( fmt, "X <- {:?}", arg ),
            LDY( arg )  => write!( fmt, "Y <- {:?}", arg ),
            LSR( arg )  => write!( fmt, "LSP {:?}", arg ),
            NOP         => write!( fmt, "NOP" ),
            ORA( arg )  => write!( fmt, "ORA {:?}", arg ),
            PHA         => write!( fmt, "PSH A" ),
            PHP         => write!( fmt, "PSH STATUS" ),
            PLA         => write!( fmt, "POP A" ),
            PLP         => write!( fmt, "POP STATUS" ),
            ROL( arg )  => write!( fmt, "ROL {:?}", arg ),
            ROR( arg )  => write!( fmt, "ROR {:?}", arg ),
            RTI         => write!( fmt, "RTI" ),
            RTS         => write!( fmt, "RTS" ),
            SBC( arg )  => write!( fmt, "SBC {:?}", arg ),
            SEC         => write!( fmt, "SET Carry" ),
            SED         => write!( fmt, "SET BCDMode" ),
            SEI         => write!( fmt, "SET IRQDisable" ),
            STA( arg )  => write!( fmt, "{:?} <- A", arg ),
            STX( arg )  => write!( fmt, "{:?} <- X", arg ),
            STY( arg )  => write!( fmt, "{:?} <- Y", arg ),
            TAX         => write!( fmt, "X <- A" ),
            TAY         => write!( fmt, "Y <- A" ),
            TSX         => write!( fmt, "X <- SP" ),
            TXA         => write!( fmt, "A <- X" ),
            TXS         => write!( fmt, "SP <- X" ),
            TYA         => write!( fmt, "A <- Y" ),
            // Unofficial instructions.
            LAX( arg )  => write!( fmt, "A, X <- {:?}", arg ),
            SAX( arg )  => write!( fmt, "{:?} <- A & X", arg ),
            AXS( arg )  => write!( fmt, "X <- (A & X) - {:?}", arg ),

            UNK( opc )  => write!( fmt, "UNK [{:02X}]", opc )
        }
    }
}

struct DecodedOpcode {
    pc: u16,       /* Address from which we decoded the instruction. */
    addr: u16,     /* RAM address of the operand; undefined if the operand is a register. */
    operand: u8,   /* Value of the operand; undefined if the instruction doesn't actually read the operand. */
    opcode: u8,
    opcode_attr: OpcodeAttributes
}

impl Deref for DecodedOpcode {
    type Target = OpcodeAttributes;

    #[inline(always)]
    fn deref<'a>( &'a self ) -> &Self::Target {
        &self.opcode_attr
    }
}

#[must_use]
fn are_on_same_page( a: u16, b: u16 ) -> bool {
    (a & 0xFF00) == (b & 0xFF00)
}

decoding_logic!();

trait Private: Sized + Context {
    /*
        ADDRESSING MODES

        Fetch the operands based on information contained in self.state().decoded.
    */

    fn memop_imm8( &mut self ) {
        debug_assert!( self.state().decoded.is_operand_fetched_from_address() == false );
    }

    fn memop_abs_zp( &mut self ) {
        self.state_mut().decoded.addr = self.state().decoded.operand as u16;
    }

    fn memop_abs( &mut self ) {
        let hi = self.fetch_and_increment_pc() as u16;

        self.state_mut().decoded.addr = (hi << 8) | (self.state().decoded.operand as u16);
    }

    fn memop_indexed_zp( &mut self ) {
        let dummy_addr = self.state().decoded.operand as u16;
        self.dummy_fetch( dummy_addr ); /* Time to add the offset. */
        self.state_mut().decoded.addr = (self.state().decoded.operand.wrapping_add( match self.state().decoded.get_register() {
            X => self.state().x,
            Y => self.state().y,
            _ => unsafe { fast_unreachable!() }
        })) as u16;
    }

    fn memop_indexed( &mut self ) {
        let hi = (self.fetch_and_increment_pc() as u16) << 8;
        let offset = match self.state().decoded.get_register() {
            X => self.state().x,
            Y => self.state().y,
            _ => unsafe { fast_unreachable!() }
        } as u16;

        let target = (hi | (self.state().decoded.operand as u16)).wrapping_add( offset );
        let unfixed_target = hi | (target & 0x00FF);

        if unfixed_target != target || self.state().decoded.is_extra_cycle_forced() {
            self.dummy_fetch( unfixed_target as Address ); /* Time to propagate carry to upper byte. */
        }

        self.state_mut().decoded.addr = target;
    }

    fn memop_indexed_indirect( &mut self ) {
        let addr_ptr_1 = self.state().decoded.operand as u16;
        let addr_ptr_2 = (self.state().decoded.operand.wrapping_add(1)) as u16;

        let lo = self.fetch( addr_ptr_1 ) as u16;
        let hi = (self.fetch( addr_ptr_2 ) as u16) << 8;
        let target = (hi | lo).wrapping_add( self.state().y as u16 );
        let unfixed_target = hi | (target & 0x00FF);

        if unfixed_target != target || self.state().decoded.is_extra_cycle_forced() {
            self.dummy_fetch( unfixed_target as Address ); /* Time to propagate carry to upper byte. */
        }

        self.state_mut().decoded.addr = target;
    }

    fn memop_indirect_indexed( &mut self ) {
        let dummy_addr = self.state().decoded.operand as u16;
        let addr = self.state().decoded.operand.wrapping_add( self.state().x );

        self.dummy_fetch( dummy_addr ); /* Time to add X to the address. */

        let lo = self.fetch( addr as u16 ) as u16;
        let hi = (self.fetch( (addr.wrapping_add(1)) as u16 ) as u16) << 8;

        self.state_mut().decoded.addr = hi | lo;
    }

    fn memop_reg( &mut self ) {
        debug_assert!( self.state().decoded.is_operand_fetched_from_address() == false );
        self.state_mut().decoded.operand = match self.state().decoded.get_register() {
             A => self.state().a,
             X => self.state().x,
             Y => self.state().y,
            SP => self.state().sp,
             _ => unsafe { fast_unreachable!() }
        }
    }

    /* INSTRUCTIONS */

    fn branch( &mut self, flag: StatusFlag, condition: bool ) -> Result< EmulationStatus, EmulationError > {
        if self.is_set( flag ) != condition {
            return Ok( EmulationStatus::Normal );
        }

        let dummy_addr = self.state().pc;
        self.dummy_fetch( dummy_addr ); /* Time to add the offset to PCL. */

        let offset = self.state().decoded.operand as i8 as i16 as u16;
        let target = self.state().pc.wrapping_add( offset );
        let current = self.state().pc;

        if !are_on_same_page( current, target ) {
            /* The upper portion is still unadjusted. */
            self.dummy_fetch( (target & 0x00FF) | ((current) & 0xFF00) ); /* Time to propagate carry. */
        }

        self.state_mut().pc = target;
        Ok( EmulationStatus::Normal )
    }

    fn set( &mut self, flag: StatusFlag ) -> Result< EmulationStatus, EmulationError > {
        self.set_flags( flag, true );
        Ok( EmulationStatus::Normal )
    }

    fn clr( &mut self, flag: StatusFlag ) -> Result< EmulationStatus, EmulationError > {
        self.set_flags( flag, false );
        Ok( EmulationStatus::Normal )
    }

    fn mov2reg( &mut self, dst: Register8 ) -> Result< EmulationStatus, EmulationError > {
        let value = self.state().decoded.operand;
        self.set_SZ( value );

        match dst {
            A => self.state_mut().a = value,
            X => self.state_mut().x = value,
            Y => self.state_mut().y = value,
            _ => unsafe { fast_unreachable!() }
        };

        Ok( EmulationStatus::Normal )
    }

    fn reg2mem( &mut self, src: Register8 ) -> Result< EmulationStatus, EmulationError > {
        let value = match src {
            A => self.state().a,
            X => self.state().x,
            Y => self.state().y,
            _ => unsafe { fast_unreachable!() }
        };

        self.write_to_operand( value );

        Ok( EmulationStatus::Normal )
    }

    fn reg2mem_and( &mut self, src_1: Register8, src_2: Register8 ) -> Result< EmulationStatus, EmulationError > {
        let value_1 = match src_1 {
            A => self.state().a,
            X => self.state().x,
            Y => self.state().y,
            _ => unsafe { fast_unreachable!() }
        };

        let value_2 = match src_2 {
            A => self.state().a,
            X => self.state().x,
            Y => self.state().y,
            _ => unsafe { fast_unreachable!() }
        };

        let value = value_1 & value_2;
        self.write_to_operand( value );

        Ok( EmulationStatus::Normal )
    }

    fn txs( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.state_mut().sp = self.state().x;
        Ok( EmulationStatus::Normal )
    }

    fn compare( &mut self, reg: Register8 ) -> Result< EmulationStatus, EmulationError > {
        let value = match reg {
            A => self.state().a,
            X => self.state().x,
            Y => self.state().y,
            _ => unsafe { fast_unreachable!() }
        };

        let diff = ( value as u16 ).wrapping_sub( self.state().decoded.operand as u16 );
        self.set_flags( StatusFlag::Carry, diff < 0x100 );
        self.set_flags( StatusFlag::Sign, (diff & 0x80) != 0 );
        self.set_flags( StatusFlag::Zero, diff == 0 );

        Ok( EmulationStatus::Normal )
    }

    fn adc( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let reg_a = self.state().a as u16;
        let op = self.state().decoded.operand as u16;
        let carry = self.is_set( StatusFlag::Carry ) as u16;

        if !Self::bcd_mode_supported() || self.is_set( StatusFlag::BCDMode ) == false {
            let result = reg_a.wrapping_add( op ).wrapping_add( carry );

            self.set_flags( StatusFlag::Zero, result as u8 == 0 );
            self.set_flags( StatusFlag::Sign, (result & 0x80) != 0 );
            self.set_flags( StatusFlag::Carry, result > 0xFF );
            self.set_flags( StatusFlag::Overflow, !(((reg_a ^ op) & 0x80) != 0) && (((reg_a ^ result) & 0x80) != 0) );
            self.state_mut().a = result as u8;
        } else {
            let mut tmp = (reg_a & 0x0f).wrapping_add( op & 0xf ).wrapping_add( carry );
            if tmp > 0x9 {
                tmp = tmp.wrapping_add( 0x6 );
            }
            if tmp <= 0x0f {
                tmp = (tmp & 0xf).wrapping_add( reg_a & 0xf0 ).wrapping_add( op & 0xf0 );
            } else {
                tmp = (tmp & 0xf).wrapping_add( reg_a & 0xf0 ).wrapping_add( op & 0xf0 ).wrapping_add( 0x10 );
            }
            self.set_flags( StatusFlag::Zero, ((reg_a.wrapping_add(op).wrapping_add(carry)) & 0xff) == 0 );
            self.set_flags( StatusFlag::Sign, (tmp & 0x80) != 0 );
            self.set_flags( StatusFlag::Overflow, ((reg_a ^ tmp) & 0x80) != 0 && ((reg_a ^ op) & 0x80) == 0 );
            if (tmp & 0x1f0) > 0x90 {
                tmp = tmp.wrapping_add(0x60);
            }
            self.set_flags( StatusFlag::Carry, (tmp & 0xff0) > 0xf0);
            self.state_mut().a = tmp as u8;
        }
        Ok( EmulationStatus::Normal )
    }

    fn sbc( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let reg_a = self.state().a as u16;
        let op = self.state().decoded.operand as u16;
        let carry = (!self.is_set( StatusFlag::Carry )) as u16;

        if !Self::bcd_mode_supported() || !self.is_set( StatusFlag::BCDMode ) {
            let result = reg_a.wrapping_sub( op ).wrapping_sub( carry );

            self.set_SZ( result as u8 );
            self.set_flags( StatusFlag::Overflow, (((reg_a ^ op) & 0x80) != 0) && (((reg_a ^ result) & 0x80) != 0) );
            self.set_flags( StatusFlag::Carry, result < 0x100 );
            self.state_mut().a = result as u8;
        } else {
            let tmp = reg_a.wrapping_sub( op ).wrapping_sub( carry );

            let mut tmp_a = (reg_a & 0xf).wrapping_sub( op & 0xf ).wrapping_sub( carry );
            if (tmp_a & 0x10) != 0 {
                tmp_a = ((tmp_a.wrapping_sub(6)) & 0xf) | ((reg_a & 0xf0).wrapping_sub(op & 0xf0).wrapping_sub(0x10));
            } else {
                tmp_a = (tmp_a & 0xf) | (reg_a & 0xf0).wrapping_sub(op & 0xf0);
            }
            if (tmp_a & 0x100) != 0 {
                tmp_a = tmp_a.wrapping_sub( 0x60 );
            }
            self.set_flags( StatusFlag::Carry, tmp < 0x100 );
            self.set_SZ( tmp as u8 );
            self.set_flags( StatusFlag::Overflow, ( ((reg_a ^ tmp) & 0x80) != 0 ) && ( ((reg_a ^ op) & 0x80) != 0 ) );
            self.state_mut().a = tmp_a as u8;
        }
        Ok( EmulationStatus::Normal )
    }

    fn and( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let result = self.state().decoded.operand & self.state().a;
        self.state_mut().a = result;
        self.set_SZ( result );

        Ok( EmulationStatus::Normal )
    }

    fn eor( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let result = self.state().decoded.operand ^ self.state().a;
        self.state_mut().a = result;
        self.set_SZ( result );

        Ok( EmulationStatus::Normal )
    }

    fn ora( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let result = self.state().decoded.operand | self.state().a;
        self.state_mut().a = result;
        self.set_SZ( result );

        Ok( EmulationStatus::Normal )
    }

    fn incdec( &mut self, diff: i8 ) -> Result< EmulationStatus, EmulationError > {
        let src = self.state().decoded.operand;
        let result = ( src as i8 ).wrapping_add( diff ) as u8;

        if !self.state().decoded.is_operand_an_register() {
            self.dummy_operand_writeback(); /* Time to modify the value. */
        }

        self.set_SZ( result );
        if self.state().decoded.is_operand_an_register() {
            match self.state().decoded.get_register() {
                X => self.state_mut().x = result,
                Y => self.state_mut().y = result,
                _ => unsafe { fast_unreachable!() }
            }
        } else {
            self.write_to_operand( result );
        }

        Ok( EmulationStatus::Normal )
    }

    fn shift( &mut self, direction: Direction ) -> Result< EmulationStatus, EmulationError > {
        let operand = self.state().decoded.operand;
        let result = match direction {
            Left => operand << 1,
            Right => operand >> 1
        };

        if !self.state().decoded.is_operand_an_register() {
            self.dummy_operand_writeback(); /* Time to modify the value. */
        }

        self.set_flags( StatusFlag::Carry, match direction {
            Left => (operand & 0x80) != 0,
            Right => (operand & 0x01) != 0
        });
        self.set_SZ( result );

        if self.state().decoded.is_operand_an_register() {
            debug_assert!( self.state().decoded.get_register() == A );
            self.state_mut().a = result as u8;
        } else {
            self.write_to_operand( result as u8 );
        }

        Ok( EmulationStatus::Normal )
    }

    fn asl( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.shift( Left )
    }

    fn lsr( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.shift( Right )
    }

    fn rotate( &mut self, direction: Direction ) -> Result< EmulationStatus, EmulationError > {
        let operand = self.state().decoded.operand;
        let carry = self.is_set( StatusFlag::Carry ) as u16;
        let result = match direction {
            Left => ((operand as u16) << 1) | carry,
            Right => ((operand as u16) >> 1) | (carry << 7)
        };

        if !self.state().decoded.is_operand_an_register() {
            self.dummy_operand_writeback(); /* Time to modify the value. */
        }

        self.set_flags( StatusFlag::Carry, match direction {
            Left => result > 0xFF,
            Right => (operand & 0x01) != 0
        });

        self.set_SZ( result as u8 );

        if self.state().decoded.is_operand_an_register() {
            debug_assert!( self.state().decoded.get_register() == A );
            self.state_mut().a = result as u8;
        } else {
            self.write_to_operand( result as u8 );
        }

        Ok( EmulationStatus::Normal )
    }

    fn rol( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.rotate( Left )
    }

    fn ror( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.rotate( Right )
    }

    fn bit( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let a = self.state().a;
        let operand = self.state().decoded.operand;

        self.set_flags( StatusFlag::Zero, (a & operand) == 0 );
        self.set_flags( StatusFlag::Sign, (operand & 0x80) != 0 );
        self.set_flags( StatusFlag::Overflow, (operand & 0x40) != 0 );

        Ok( EmulationStatus::Normal )
    }

    fn jmp_imm( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let lo = self.state().decoded.operand as u16;
        let hi = (self.fetch_pc() as u16) << 8;
        let target = hi | lo;
        self.state_mut().pc = target;

        if target == self.state().decoded.pc {
            Ok( EmulationStatus::InfiniteLoop( self.state().decoded.pc ) )
        } else {
            Ok( EmulationStatus::Normal )
        }
    }

    fn jmp_abs( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let lo = self.state().decoded.operand as u16;
        let hi = (self.fetch_and_increment_pc() as u16) << 8;
        let ptr = hi | lo;

        /* PCH is always fetched from the same page as PCL. */
        let pcl = self.fetch( ptr ) as u16;
        let pch = (self.fetch( ((ptr.wrapping_add(1)) & 0x00FF) | (ptr & 0xFF00) ) as u16) << 8;
        let target = pcl | pch;
        self.state_mut().pc = target;

        if target == self.state().decoded.pc {
            Ok( EmulationStatus::InfiniteLoop( self.state().decoded.pc ) )
        } else {
            Ok( EmulationStatus::Normal )
        }
    }

    fn jsr( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let lo = self.state().decoded.operand as u16;
        let hi_ptr = self.state().pc;

        let dummy_addr = self.stack_address(); /* Time to load stack address into ALU and put the operand in SP. (Verified in visual6502.) */
        self.dummy_fetch( dummy_addr );
        self.push_pc();

        let hi = self.fetch( hi_ptr );
        let target = lo | ((hi as u16) << 8);
        self.state_mut().pc = target;

        Ok( EmulationStatus::Normal )
    }

    /* See: http://wiki.nesdev.com/w/index.php/CPU_interrupts */
    fn hardware_interrupt( &mut self ) {
        self.set_flags( StatusFlag::SoftIRQ, false );
        self.push_pc();

        /* Interrupt hijacking. */
        let vector_address =
            if self.state().nmi_pending {
                /* Trigger as NMI. */
                NMI_VECTOR_ADDRESS
            } else {
                /* Trigger as IRQ. */
                IRQ_VECTOR_ADDRESS
            };

        let status = self.state().status;
        self.push( status );
        self.set_flags( StatusFlag::IRQDisable, true );

        let pcl = self.fetch( vector_address );
        self.set_pcl( pcl );

        let pch = self.fetch( vector_address + 1 );
        self.set_pch( pch );
    }

    fn brk( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.set_flags( StatusFlag::SoftIRQ, true );
        self.push_pc();

        /* Interrupt hijacking. */
        let vector_address = if self.state().nmi_latch_state {
            NMI_VECTOR_ADDRESS
        } else {
            IRQ_VECTOR_ADDRESS
        };

        let status = self.state().status;
        self.push( status );

        self.set_flags( StatusFlag::IRQDisable, true );

        let pcl = self.fetch( vector_address );
        self.set_pcl( pcl );

        let pch = self.fetch( vector_address + 1 );
        self.set_pch( pch );

        Ok( EmulationStatus::Normal )
    }

    fn pha( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let value = self.state().a;
        self.push( value );

        Ok( EmulationStatus::Normal )
    }

    fn php( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.set_flags( StatusFlag::SoftIRQ, true );
        let value = self.state().status;
        self.push( value );

        Ok( EmulationStatus::Normal )
    }

    fn pla( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.dummy_pop();

        let a = self.fetch_stack();
        self.state_mut().a = a;
        self.set_flags( StatusFlag::Sign, (a & 0x80) != 0 );
        self.set_flags( StatusFlag::Zero, a == 0 );

        Ok( EmulationStatus::Normal )
    }

    fn plp( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.dummy_pop(); // A dummy pop to increment the stack pointer.

        let status = self.fetch_stack();
        self.state_mut().status = status;
        self.set_flags( StatusFlag::Unused, true );

        Ok( EmulationStatus::Normal )
    }

    fn rti( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.dummy_pop(); // A dummy pop to increment the stack pointer.

        let status = self.pop();
        self.state_mut().status = status;
        self.set_flags( StatusFlag::Unused, true );
        self.pop_pc();

        Ok( EmulationStatus::Normal )
    }

    fn rts( &mut self ) -> Result< EmulationStatus, EmulationError > {
        self.dummy_pop(); // A dummy pop to increment the stack pointer.
        self.pop_pc();
        let _ = self.fetch_and_increment_pc();

        Ok( EmulationStatus::Normal )
    }

    fn nop( &mut self ) -> Result< EmulationStatus, EmulationError > {
        Ok( EmulationStatus::Normal )
    }

    fn unk( &mut self ) -> Result< EmulationStatus, EmulationError > {
        Err( EmulationError::InvalidInstruction( self.state().pc, self.state().decoded.opcode ) )
    }

    /* UNOFFICIAL INSTRUCTIONS */

    fn axs( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let value = self.state().a & self.state().x;
        let diff = ( value as u16 ).wrapping_sub( self.state().decoded.operand as u16 );

        self.set_flags( StatusFlag::Carry, diff < 0x100 );
        self.set_SZ( diff as u8 );
        self.state_mut().x = diff as u8;

        Ok( EmulationStatus::Normal )
    }

    /* OTHER */
    fn reset( &mut self ) {
        self.dummy_fetch( 0x00FF );
        self.dummy_fetch( 0x00FF );
        self.dummy_fetch( 0x00FF );
        self.dummy_fetch( 0x0100 );
        self.dummy_fetch( 0x01FF );
        self.dummy_fetch( 0x01FE );

        self.state_mut().sp = 0xFD;
        let lo = self.fetch( RESET_VECTOR_ADDRESS ) as u16;
        let hi = (self.fetch( RESET_VECTOR_ADDRESS + 1 ) as u16) << 8;
        self.state_mut().pc = lo | hi;
    }

    fn set_nmi_latch( &mut self, state: bool ) {
        self.state_mut().nmi_latch_state = state;
    }

    fn set_irq_line( &mut self, state: bool ) {
        self.state_mut().irq_line_state = state;
    }

    #[must_use]
    fn irq_line( &self ) -> bool {
        self.state().irq_line_state
    }

    fn execute( &mut self ) -> Result< EmulationStatus, EmulationError > {
        let nmi_pending = self.state().nmi_latch_state;

        self.state_mut().decoded.pc = self.state().pc;
        self.state_mut().decoded.opcode = self.fetch_pc();

        {
            let pc = self.state_mut().decoded.pc;
            let opcode = self.state().decoded.opcode;
            self.trace( pc, opcode );
        }

        if nmi_pending && !self.state().nmi_pending {
            self.state_mut().nmi_pending = true;

            let address = self.state().pc;
            self.dummy_fetch( address );
            self.hardware_interrupt();

            return Ok( EmulationStatus::Normal );
        } else if self.state().irq_line_state && !self.is_set( StatusFlag::IRQDisable ) {
            let address = self.state().pc;
            self.dummy_fetch( address );
            self.hardware_interrupt();

            return Ok( EmulationStatus::Normal );
        }

        if !nmi_pending {
            self.state_mut().nmi_pending = false;
        }

        self.state_mut().pc.wrapping_inc();
        self.state_mut().decoded.operand = self.fetch_pc();
        self.state_mut().decoded.opcode_attr = OpcodeAttributes::get( self.state().decoded.opcode );

        if !self.state().decoded.is_single_byte_instruction() {
            self.state_mut().pc.wrapping_inc();
        }

        if !self.state().decoded.is_single_byte_imm8() {
            let execute_memop = self.state().decoded.get_memop();
            execute_memop( self );

            if self.state().decoded.is_operand_fetched_from_address() {
                let addr = self.state().decoded.addr;
                self.state_mut().decoded.operand = self.fetch( addr );
            }
        }

        let result = instruction_for_opcode( self.state().decoded.opcode )( self );
        result
    }

    #[cfg(feature = "std")]
    fn print_registers( &self ) {
        println!( "            NV-BDIZC  A  X  Y   PC" );
        println!( "            {:08b} {:02X} {:02X} {:02X} {:04X}", self.state().status, self.state().a, self.state().x, self.state().y, self.state().pc );
    }

    #[must_use]
    fn get_register( &self, register: Register8 ) -> u8 {
        use self::Register8::*;
        match register {
                 A => self.state().a,
                 X => self.state().x,
                 Y => self.state().y,
                SP => self.state().sp,
            STATUS => self.state().status
        }
    }

    #[must_use]
    fn fetch( &mut self, address: Address ) -> u8 {
        self.peek( address )
    }

    #[inline]
    fn dummy_fetch( &mut self, address: Address ) {
        let _ = self.fetch( address );
    }

    fn write( &mut self, address: Address, value: u8 ) {
        self.poke( address, value );
    }

    #[inline]
    fn set_flags( &mut self, flags: StatusFlag, condition: bool ) {
        let status = self.state().status;
        let new_status = if condition { status | flags.bits() } else { status & !flags.bits() };

        self.state_mut().status = new_status;
    }

    #[inline]
    fn is_set( &self, flags: StatusFlag ) -> bool {
        (self.state().status & flags.bits()) == flags.bits()
    }

    #[inline]
    fn set_SZ( &mut self, value: u8 ) {
        self.set_flags( StatusFlag::Sign, (value & 0x80) != 0 );
        self.set_flags( StatusFlag::Zero, value == 0 );
    }

    #[inline]
    fn set_pcl( &mut self, value: u8 ) {
        unsafe {
            let ppc: *mut u8 = (&mut self.state_mut().pc) as *mut u16 as *mut u8;
            *ppc = value;
        }
    }

    #[inline]
    fn set_pch( &mut self, value: u8 ) {
        unsafe {
            let ppc: *mut u8 = (&mut self.state_mut().pc) as *mut u16 as *mut u8;
            *(ppc.add( 1 )) = value;
        }
    }

    #[must_use]
    #[inline]
    fn stack_address( &self ) -> u16 {
        (self.state().sp as u16).wrapping_add( 0x100 )
    }

    #[inline]
    fn fetch_stack( &mut self ) -> u8 {
        let address = self.stack_address();
        self.fetch( address )
    }

    #[inline]
    fn write_stack( &mut self, value: u8 ) {
        let address = self.stack_address();
        self.write( address, value );
    }

    #[inline]
    fn write_to_operand( &mut self, value: u8 ) {
        debug_assert!( self.state().decoded.is_operand_an_register() == false );

        let address = self.state().decoded.addr;
        self.write( address, value );
    }

    #[inline]
    fn dummy_operand_writeback( &mut self ) {
        let value = self.state().decoded.operand;
        self.write_to_operand( value );
    }

    fn push( &mut self, value: u8 ) {
        self.write_stack( value );
        self.state_mut().sp.wrapping_dec();
    }

    #[must_use]
    fn pop( &mut self ) -> u8 {
        let value = self.fetch_stack();
        self.state_mut().sp.wrapping_inc();
        value
    }

    #[inline]
    fn dummy_pop( &mut self ) {
        let _ = self.pop();
    }

    fn push_pc( &mut self ) {
        let hi = (self.state().pc >> 8) as u8;
        self.push( hi );

        let lo = self.state().pc as u8;
        self.push( lo );
    }

    fn pop_pc( &mut self ) {
        let pcl = self.pop();
        self.set_pcl( pcl );

        let pch = self.fetch_stack();
        self.set_pch( pch );
    }

    #[must_use]
    fn fetch_and_increment_pc( &mut self ) -> u8 {
        let value = self.fetch_pc();
        self.state_mut().pc.wrapping_inc();
        value
    }

    #[must_use]
    fn fetch_pc( &mut self ) -> u8 {
        let address = self.state().pc;
        self.fetch( address )
    }
}
