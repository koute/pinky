#!/usr/bin/ruby
#
# This script is used to generate virtual_mos6502_decoder.rs based on mos6502_opcodes.dat.
#

def output( code = '' )
    $output ||= ""
    $indentation = 0 if $indentation == nil
    lines = code.strip.split( "\n" ).map do |line|
        balanced_braces = line.count("{") == line.count("}")

        if line.strip.end_with?( '}' ) || line.strip.end_with?( '];' ) || (balanced_braces == false && line.strip.end_with?( '},' ))
            $indentation -= 4
        end
        line = (' ' * $indentation) + line.strip
        if line.end_with?( '{' ) || line.end_with?( '[' )
            $indentation += 4
        end

        line
    end

    $output << lines.join( "\n" )
    $output << "\n"
end

def write_output_to_file( filename )
    STDERR.puts "Generating #{filename}..."
    File.open( get_path( filename ), "wb" ) do |fp|
        fp.puts $output
    end
end

def get_path( filename )
    File.join( File.dirname( __FILE__ ), filename )
end

def read_lines( filename )
    File.read( filename ).strip.split( "\n" ).map( &:strip ).select { |line| !line.start_with?( '#' ) && !line.empty? }
end

class Instruction < Struct.new( :opcode, :rw, :mnemonic, :memop, :code )

    def single?
        INSTRUCTIONS.count { |instr| instr.mnemonic == self.mnemonic } == 1
    end

    def implied_addressing?
        is_single = self.single?
        !((self.disassembly_memop != '_') && (!is_single || (is_single && !self.disassembly_memop.start_with?('reg_'))))
    end

    def emulation_memop
        self.memops[0]
    end

    def disassembly_memop
        self.memops[1]
    end

    def memops
        if self.memop =~ /\A([^\|]+)\|(.*)\Z/
            [$1, $2]
        else
            [self.memop, self.memop]
        end
    end

    def callback_name
        "exec_#{self.code.gsub( /;| |\(\)|(\)$)/, '' ).gsub( /\(|\)|,|::/, '_' ).gsub( '-', 'neg_' ).downcase}"
    end

end

def read_data( filename )

    instructions = read_lines( filename ).map do |line|
        throw "wtf: #{line}" unless line =~ /0x(..)\s+(R|RW|W|\?|_)\s+(\S+)\s+(\S+)\s+(.+)/
        Instruction.new $1.to_i( 16 ), $2, $3, $4, $5
    end

    instruction_map = instructions.map { |instr| [instr.opcode, instr] }.to_h
    [instructions, instruction_map]

end

def generate_opcode_enum

    output <<-EOS
        #[derive(Copy, Clone, PartialEq)]
        pub enum Opcode {
    EOS

    INSTRUCTIONS.uniq( &:mnemonic ).map do |instruction|

        name = instruction.mnemonic.upcase

        # There is no point in including an argument if the instruction uses implied addressing.
        arg = unless instruction.implied_addressing?
            memop = instruction.disassembly_memop.to_sym
            is_single = instruction.single?

            if is_single && memop == :imm8
                '( u8 )'
            elsif is_single && memop == :imm16
                '( u16 )'
            else
                '( Location )'
            end
        else
            ''
        end

        output "#{name}#{arg},"

    end

    output <<-EOS
            UNK( u8 )
        }
    EOS

end

def generate_opcode_decoder

    output
    output <<-EOS
        pub fn decode_instruction( opcode: u8, arg_lo: u8, arg_hi: u8 ) -> Opcode {
            use self::Location::*;
            use self::Opcode::*;

            let arg16 = (arg_lo as u16) | ((arg_hi as u16) << 8);
            match opcode {
    EOS
    0.upto( 0xFF ).each do |opcode|

        next unless INSTRUCTION_MAP.has_key? opcode

        instr = INSTRUCTION_MAP[ opcode ]
        memop = instr.disassembly_memop
        args = if instr.implied_addressing?
            ''
        else
            case memop.to_sym
                when :imm8
                    if instr.single?
                        '( arg_lo )'
                    else
                        '( Imm8( arg_lo ) )'
                    end
                when :imm16
                    if instr.single?
                        '( arg16 )'
                    else
                        '( Imm16( arg16 ) )'
                    end
                when :abs
                    '( Abs( arg16 ) )'
                when :abs_zp
                    '( AbsZP( arg_lo ) )'
                when :addx_zp
                    '( IndexedZP( X, arg_lo ) )'
                when :addy_zp
                    '( IndexedZP( Y, arg_lo ) )'
                when :addx, :addx_EC
                    '( Indexed( X, arg16 ) )'
                when :addy, :addy_EC
                    '( Indexed( Y, arg16 ) )'
                when :indexed_indirect, :indexed_indirect_EC
                    '( IndexedIndirect( arg_lo ) )'
                when :indirect_indexed, :indirect_indexed_EC
                    '( IndirectIndexed( arg_lo ) )'
                when :reg_a
                    '( Reg( A ) )'
                when :reg_x
                    '( Reg( X ) )'
                when :reg_y
                    '( Reg( Y ) )'
                when :reg_sp
                    '( Reg( SP ) )'
                else
                    throw "wtf: '#{memop}' for #{instr}"
            end
        end

        output "0x%02X => #{instr.mnemonic.upcase}#{args}," % [ opcode ]

    end

    output <<-EOS
                _ => UNK( opcode )
            }
        }
    EOS

end

def generate_opcode_jump_table

    output
    output <<-EOS
        fn exec_unk< T: Private >( cpu: &mut T ) -> Result< EmulationStatus, EmulationError > {
            cpu.unk()
        }
    EOS

    done = {}
    INSTRUCTIONS.each do |instruction|

        name = instruction.callback_name
        next if done.has_key? name
        done[ name ] = true

        lines = instruction.code.split( /;\s*/ )
        0.upto( lines.length - 2 ) { |i| lines[ i ] += "?;" }

        code = lines.map { |line| "cpu.#{line}" }.join( "\n" + " " * 16 )

        output
        output <<-EOS
            fn #{name}< T: Private >( cpu: &mut T ) -> Result< EmulationStatus, EmulationError > {
                #{code}
            }
        EOS

    end

    output
    output <<-EOS
        fn instruction_for_opcode< T: Private >( opcode: u8 ) -> fn( &mut T ) -> Result< EmulationStatus, EmulationError > {
            match opcode {
    EOS

    0.upto( 0xFF ).each do |opcode|

        line = if INSTRUCTION_MAP.has_key? opcode
            "#{opcode} => " + INSTRUCTION_MAP[ opcode ].callback_name + ","
        else
            "#{opcode} => exec_unk,"
        end
        output line

    end

    output <<-EOS
                _ => unsafe {
                    fast_unreachable!()
                }
            }
        }
    EOS

end

# Opcode attribute encoding:
#   76543210
#   IFERRAAA
#   I -> is this a single byte instuction? (prevents PC++)
#   F -> is the operand fetched from the address
#   E -> is the extra cycle forced in indexed and indirect_indexed addressing modes
#   R -> a register
#           00 -> A
#           01 -> X
#           10 -> Y
#           11 -> SP
#   A -> addressing mode
#           000 -> imm8
#           001 -> abs
#           010 -> abs_zp
#           011 -> indexed_zp
#           100 -> indexed
#           101 -> indexed_indirect
#           110 -> indirect_indexed
#           111 -> reg
ADDRMODE_BITPATTERN = {
    :imm8                => 0b00000000,
    :abs                 => 0b00000001,
    :abs_zp              => 0b00000010,
    :addx_zp             => 0b00001011,
    :addy_zp             => 0b00010011,
    :addx                => 0b00001100,
    :addy                => 0b00010100,
    :addx_EC             => 0b00101100,
    :addy_EC             => 0b00110100,
    :indexed_indirect    => 0b00000101,
    :indexed_indirect_EC => 0b00100101,
    :indirect_indexed    => 0b00000110,
    :indirect_indexed_EC => 0b00100110,
    :reg_a               => 0b00000111,
    :reg_x               => 0b00001111,
    :reg_y               => 0b00010111,
    :reg_sp              => 0b00011111
}

def generate_opcode_attributes

    output
    output <<-EOS
        #[derive(Copy, Clone, PartialEq, Eq)]
        struct OpcodeAttributes {
            attr: u8
        }

        impl OpcodeAttributes {
            const fn empty() -> OpcodeAttributes {
                OpcodeAttributes {
                    attr: 0
                }
            }

            fn get( opcode: u8 ) -> OpcodeAttributes {
                OpcodeAttributes {
                    attr: unsafe {
                        *OPCODE_ATTRIBUTE_TABLE.get_unchecked( opcode as usize )
                    }
                }
            }

            #[inline(always)]
            fn is_single_byte_imm8( &self ) -> bool {
                self.attr == 0b10000000
            }

            #[inline(always)]
            fn is_operand_an_register( &self ) -> bool {
                (self.attr & 0b00000111) == 0b00000111
            }

            #[inline(always)]
            fn is_single_byte_instruction( &self ) -> bool {
                (self.attr & 0b10000000) != 0
            }

            #[inline(always)]
            fn is_operand_fetched_from_address( &self ) -> bool {
                (self.attr & 0b01000000) != 0
            }

            /* Whenever the extra cycle is always forced in indexed and indirect_indexed addressing modes. */
            #[inline(always)]
            fn is_extra_cycle_forced( &self ) -> bool {
                (self.attr & 0b00100000) != 0
            }

            #[inline(always)]
            fn get_register( &self ) -> Register8 {
                unsafe {
                    core::mem::transmute( (self.attr & 0b00011000) >> 3 )
                }
            }

            #[inline(always)]
            fn get_memop< T: Private >( &self ) -> fn( &mut T ) {
                match self.attr & 0b00000111 {
                    0 => T::memop_imm8,
                    1 => T::memop_abs,
                    2 => T::memop_abs_zp,
                    3 => T::memop_indexed_zp,
                    4 => T::memop_indexed,
                    5 => T::memop_indexed_indirect,
                    6 => T::memop_indirect_indexed,
                    7 => T::memop_reg,
                    _ => unsafe {
                        fast_unreachable!()
                    }
                }
            }
        }

    EOS

    output
    output "const OPCODE_ATTRIBUTE_TABLE: &'static [u8] = &["

    0.upto( 0xFF ).each_slice( 8 ).map do |opcodes|

        out = opcodes.map do |opcode|

            instr = INSTRUCTION_MAP[ opcode ] if INSTRUCTION_MAP.has_key? opcode
            instr = nil if instr && instr.emulation_memop == '_'

            mode = if instr != nil
                mode = ADDRMODE_BITPATTERN[ instr.emulation_memop.to_sym ]
                mode |= 0b01000000 if (instr.rw == 'R' || instr.rw == 'RW') && !instr.emulation_memop.start_with?( 'imm' ) && !instr.emulation_memop.start_with?( 'reg_' )
                mode |= 0b10000000 if instr.memop.start_with? 'reg_'
                mode
            else
                0b10000000
            end

            "0x%02X, " % [ mode ]

        end.join( "" )

        output out

    end

    output "];"

end

DATA_FILENAME = get_path( 'mos6502_opcodes.dat' )
INSTRUCTIONS, INSTRUCTION_MAP = read_data( DATA_FILENAME )

$indentation = 0

output "// WARNING: This file was autogenerated; DO NOT EDIT."
output
output <<-EOS
    #![macro_use]
    macro_rules! decoding_logic {
        () => {
EOS
output

generate_opcode_enum()
generate_opcode_decoder()
generate_opcode_jump_table()
generate_opcode_attributes()

output "}"
output "}"

write_output_to_file "virtual_mos6502_decoder.rs"
