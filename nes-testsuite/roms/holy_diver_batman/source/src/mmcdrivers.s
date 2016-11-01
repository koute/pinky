;
; Test drivers for basic ASIC mappers for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/nes.h"
.include "src/ram.h"

; MMC1 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

.segment "DRIVER_MMC1"
  ; PRG RAM bank (for SXROM) and CHR bank (for CHR ROM) use same reg
  ; PSS00 (SUROM, SXROM): Select 256K PRG ROM and 8K CHR ROM bank
  ; CCCC0 (others): Select 8K CHR RAM bank
  and #$03
  asl a
  jmp mmc1_set_chr_8k
.proc mmc1_test_mapper

  lda #0
  sta PPUCTRL  ; turn off nmi while testing 32K bank mode

  ; Read all PRG ROM bank tags through both windows
  ldx #$80
  jsr mmc1_test_one_prg_window
  sta driver_prg_result
  ldx #$C0
  jsr mmc1_test_one_prg_window
  asl a
  ora driver_prg_result
  sta driver_prg_result

  ; Verify PRG RAM protection
  ; There are two layers of protection.  The MMC1 layer ($E000.D4 =
  ; disable) works only for MMC1B and later.  The SNROM layer
  ; ($A000.D4 = disable) works only on boards with no CHR RAM and
  ; 256K or smaller PRG ROM.
  jsr mmc1_test_wram_protection
  ora driver_prg_result
  sta driver_prg_result

  ; Read all CHR bank tags through each 4K CHR window
  ldx #$00
  jsr mmc1_test_one_chr_window
  sta driver_chr_result
  ldx #$10
  jsr mmc1_test_one_chr_window
  asl a
  ora driver_chr_result
  sta driver_chr_result
  
  ; No IRQs here
  lda #$08
.endproc
.proc mmc1_set_chr_8k
  ; For 512K boards (SUROM and SXROM) only, force CHR A16
  ; (which controls PRG A18) on
  ldy last_prg_bank
  cpy #256/4
  bcc :+
  ora #$08  ; compensate for SUROM/SXROM
:
  and #$0F
  asl a
.endproc
.proc mmc1_set_chr_0000
  ldy #5
loop:
  sta MMC1_CHRBANK0
  lsr a
  dey
  bne loop
  rts
.endproc

.proc mmc1_set_chr_1000
  ldy #5
loop:
  sta MMC1_CHRBANK1
  lsr a
  dey
  bne loop
  rts
.endproc

.proc mmc1_set_prg
  ldy #5
loop:
  sta MMC1_PRGBANK
  lsr a
  dey
  bne loop
  rts
.endproc

.proc mmc1_set_mode
  ldy #5
loop:
  sta MMC1_MODE
  lsr a
  dey
  bne loop
  rts
.endproc

.proc mmc1_test_wram_protection
protwarnings = 0

  lda has_wram
  bne :+
  rts
:

  ; turn off all prot
  lda #$00
  sta protwarnings
  jsr mmc1_set_mode
  lda #$0F
  jsr mmc1_set_prg
  lda #$00
  jsr mmc1_set_chr_0000

  ; Establish WRAM as writable
  ldx #$80
  lda #$6B
  sta $6000
  stx $E000
  cmp $6000
  beq :+
  lda #MAPTEST_WRAMRO
  rts
:

  ; Test MMC1B/MMC1C protection
  ; $E000 bit 4: disable WRAM
  lda #$1F
  jsr mmc1_set_prg
  ldx #$80
  stx $6000
  lda #$0F
  jsr mmc1_set_prg
  lda #$6B
  cmp $6000
  beq :+
  sta $6000
  lda #MAPTEST_WRAMEN
  ora protwarnings
  sta protwarnings
:

  ; Test SNROM protection
  ; $A000 bit 4: disable WRAM
  lda #$10
  jsr mmc1_set_chr_0000
  lda #$80
  sta $6000
  lda #$00
  jsr mmc1_set_chr_0000
  ; Should be present on SNROM (CHR RAM, <=256K PRG ROM)
  ; and no other boards
  lda #$80
  ldy is_chrrom
  bne snromprot_have_expected
  ldy last_prg_bank
  cpy #64
  bcs snromprot_have_expected
  lda #$6B
snromprot_have_expected:
  cmp $6000
  beq :+
  lda #MAPTEST_WRAMEN2
  ora protwarnings
  sta protwarnings
:
  lda protwarnings
  rts
.endproc

.proc mmc1_test_one_prg_window
tagsrclo = $00
tagsrchi = $01
mode = $04
prgbank = $05
tagbase = $06

  lda #$0C
  jsr mmc1_set_mode
  lda #$10
  jsr mmc1_set_chr_0000
  lda #<CUR_BANK
  sta tagsrclo
  lda #$0C
modeloop:
  sta mode
  jsr mmc1_set_mode
  lda #0
prgbankloop:
  sta prgbank

  ; Switch to the corresponding MMC1 bank
  lsr a
  lsr a
  pha
  and #$0F
  jsr mmc1_set_prg
  pla
  and #$10
  jsr mmc1_set_chr_0000

  ; Calculate which PRG ROM banks should be here
  lda prgbank
  ldy mode
  jsr mmc1_calc_prgbank
  and last_prg_bank
  sta tagbase

  ; Set up source address
  txa
  ora #>(CUR_BANK & $8FFF)
  sta tagsrchi
  ldy #0

  ; Compare all source bank tags to the expected ones
tagloop:
  lda tagbase
  cmp (tagsrclo),y
  beq tagok
  lda #1
  rts
tagok:
  inc tagbase
  lda tagsrchi
  clc
  adc #$10
  sta tagsrchi
  and #%00110000
  bne tagloop

  ; Move to the next PRG bank value
  lda prgbank
  clc
  adc #4
  bcs :+
  cmp last_prg_bank
  bcc prgbankloop
:
  lda mode
  sbc #4
  bcs modeloop
  lda #0
  rts
.endproc

;;
; Calculate the first PRG bank tag in a 16K bank given MMC1 regs.
; @param A MMC1 prgbank value shl 2 (0-127)
; @param X high byte of window ($80 or $C0)
; @param Y mode ($00, $04, $08, or $0C)
; @return the first PRG bank tag
.proc mmc1_calc_prgbank
tmp = $01
  and #$FC
  cpy #$08
  bcs is_fixed_bank_mode
  and #$F8
  cpx #$C0
  bcc :+
  ora #$04
:
  rts
is_fixed_bank_mode:
  cpy #$0C
  bcs is_fixed_C000
  ; Fixed $A000
  cpx #$C0
  bcs :+
  and #$C0
:
  rts
is_fixed_C000:
  cpx #$C0
  bcc :+
  ora #$3C
:
  rts
.endproc

.proc mmc1_test_one_chr_window
mmc1portlo = 0
mmc1porthi = 1
tagsrchi = $04
chrbank = $05
tagbase = $06
last_prg_bank = $07

  lda #0
  sta mmc1portlo
  txa
  asl a
  adc #$A0
  sta mmc1porthi

  lda #$1C
  jsr mmc1_set_mode
  lda last_chr_bank
  sec
  rol a
  asl a
  asl a
chrbankloop:
  sta chrbank
  sta tagbase

  ; Switch to the corresponding MMC1 bank
  lsr a
  lsr a
  ldy #5
:
  sta (mmc1portlo),y
  lsr a
  dey
  bne :-

  ; Set up source address
  txa
  ora #$01
  sta tagsrchi
  ldy #0

  ; Compare all source bank tags to the expected ones
tagloop:
  lda tagsrchi
  sta PPUADDR
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  lda tagbase
  cmp PPUDATA
  beq tagok
  lda #1
  rts
tagok:
  inc tagbase
  lda tagsrchi
  clc
  adc #$04
  sta tagsrchi
  and #%00001100
  bne tagloop

  ; Move to the next PRG bank value
  lda chrbank
  sec
  sbc #4
  bcs chrbankloop
  lda #0
  rts
.endproc


; INL-ROM A53 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

.segment "DRIVER_A53"
  rts  ; A53 doesn't support PRG RAM banking
  nop
  nop
  jmp a53_set_chr_8k
.proc a53_test_mapper
  ; like the MMC1, it's flexible enough to need NMIs turned off
  ; because testing it takes longer than one vblank
  lda #$00
  sta PPUCTRL

  ; Essentially we reproduce the test28 logic
  ldx #$80
  jsr a53_test_one_prg_window
  sta driver_prg_result
  ldx #$C0
  jsr a53_test_one_prg_window
  asl a
  ora driver_prg_result
  sta driver_prg_result

  lda #$00
  sta driver_chr_result
  ; No non-8K CHR banking, PRG RAM protection, or IRQ
  ; so fall through and finish
.endproc
.proc a53_set_last_bank
  ; Get the number of the last bank in the cart
  lda #$80
  sta A53_SELECT
  asl a
  sta A53_DATA
  lda #$81
  sta A53_SELECT
  lda #$FF
  sta A53_DATA
  rts
.endproc

.proc a53_set_chr_8k
  ldy #$00
  sty A53_SELECT
  and #$0F
  sta A53_DATA
  rts
.endproc

;;
; Determines what 16 KiB PRG ROM bank ought to be mapped into a given
; CPU address with a given set of $80, $81, $00 values.
; For use in a test ROM that verifies mapper 28.
; From http://wiki.nesdev.com/w/index.php/User:Tepples/Multi-discrete_mapper/Reference_implementations
; @param A $80 value
; @param X $01 value
; @param Y $81 value
; @param C CPU A14 value: clear for $8000-$BFFF, set for $C000-$FFFF
; @return PRG bank number in A
.proc a53_calc_prg_bank
bank_mode = 0
outer_bank = 1
current_bank = 2
  stx current_bank
  sty outer_bank
  rol outer_bank
  lsr a  ; discard mirroring bits
  lsr a
  sta bank_mode

  ; If the mode is UxROM (10 = mapper 180, 11 = mapper 2), and bit 0
  ; of the mode matches CPU A14, then the read is within the fixed
  ; bank.  For such reads, the mapper acts in 32K (NROM) mode.
  and #$02
  beq not_unrom
  lda outer_bank
  eor bank_mode
  ; If bit 0 of the eor result is false, there is a match, so
  ; fall through to the not-UNROM code.
  and #$01
  bne have_current_bank
  sta bank_mode

not_unrom:
  ; In 32K switched modes (NROM, CNROM, BNROM, AOROM),
  ; shift CPU A14 into the current bank
  lda outer_bank
  lsr a
  rol current_bank
  
have_current_bank:
  lda bank_mode
  lsr a
  lsr a
  and #$03
  tax
  lda current_bank
  eor outer_bank
  and a53_bank_size_masks,x
  eor outer_bank
  rts
.endproc

a53_bank_size_masks: .byt $01, $03, $07, $0F

;;
; Tests all PRG bank modes
; @param X the window to test, $80 or $C0
.proc a53_test_one_prg_window
value80 = $03
value81 = $04
value01 = $05
tagsrclo = $06
tagsrchi = $07
wndbasehi = $08

  txa
  ora #>(CUR_BANK & $8FFF)
  sta wndbasehi

  jsr a53_set_last_bank
  lda #<CUR_BANK
  sta tagsrclo
  
  lda #$3C
loop80:
  sta value80
  ldy #$80
  sty A53_SELECT
  sta A53_DATA
  ldy #$1F
loop81:
  sty value81
  lda #$81
  sta A53_SELECT
  sty A53_DATA
  sty $4011
  lda #$01
  sta A53_SELECT
  ldx #$0F
loop01:
  stx value01    ; X is already $01 value
  stx A53_DATA
  lda wndbasehi
  sta tagsrchi
  cmp #$C0       ; C = window number
  lda value80
  ldy value81
  jsr a53_calc_prg_bank
  ; A = expected value in 16K banks on 2 MB ROM
  asl a
  asl a
  and last_prg_bank
  tax            ; X = expected value in 4K banks on this ROM
  ldy #0

tagloop:
  txa
  cmp (tagsrclo),y
  beq tagok
  lda #MAPTEST_PRGWIN1
  rts
tagok:
  inx
  lda tagsrchi
  clc
  adc #$10
  sta tagsrchi
  and #%00110000
  bne tagloop
  
  ldx value01
  dex
  bpl loop01
  ldy value81
  dey
  bpl loop81
  lda value80
  sec
  sbc #4
  bpl loop80
  lda #0
  rts
.endproc

; MMC2 and MMC4 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

.segment "DRIVER_MMC2"
  rts  ; MMC2/MMC4 doesn't support PRG RAM banking
  nop
  nop
  jmp mmc2_set_chr_8k
.proc mmc2_test_mapper
  jsr mmc2_test_prg_rom
  sta driver_prg_result

  ldx #0
  stx driver_chr_result
  ldx #3
loop:
  jsr mmc2_test_one_chr_window
  ora driver_chr_result
  sta driver_chr_result
  dex
  bpl loop

  ; No PRG RAM protection or IRQ
  rts
.endproc
.proc mmc2_set_chr_8k
  asl a
  sta MMC2_CHRBANK0_FD
  sta MMC2_CHRBANK0_FE
  ora #1
  sta MMC2_CHRBANK1_FD
  sta MMC2_CHRBANK1_FE
  rts
.endproc

.proc mmc2_test_prg_rom
tagsrclo = $00
tagsrchi = $01
prgbankend = $02
prgbank = $03

  lda #<CUR_BANK
  sta tagsrclo
  lda #$A0
  ldy cur_mapper
  cpy #MAPPER_MMC4
  bne :+
  lda #$C0
:
  sta prgbankend
  ldy #0
  sty prgbank
  ldx #0   ; X = PRG bank

bankloop:
  lda #>(CUR_BANK & $8FFF)
  sta tagsrchi
  stx MMC2_PRGBANK
  inx
tagloop:
  lda (tagsrclo),y
  cmp prgbank
  beq not_testfailure
  lda #MAPTEST_PRGWIN1
  rts
not_testfailure:
  inc prgbank
  lda tagsrchi
  clc
  adc #$10
  sta tagsrchi
  cmp prgbankend
  bcc tagloop
  lda prgbank
  cmp CUR_BANK
  bcc bankloop
  lda #0
  rts
.endproc

;;
; @param X window to test (0-3)
.proc mmc2_test_one_chr_window
wndreg_lo   = $00
wndreg_hi   = $01
cur_tile_hi = $02
cur_bank    = $03

  lda #<MMC2_CHRBANK0_FD
  sta wndreg_lo
  lda mmc2_window_reg,x
  sta wndreg_hi
  lda last_chr_bank
  asl a
  asl a
  asl a
  ora #$07
  sta cur_bank
loop:
  ; Calculate address of tile in question
  lda cur_bank
  and #$03
  asl a
  asl a
  ora mmc2_base_high,x
  sta cur_tile_hi

  lda cur_bank
  lsr a
  lsr a
  eor #$3F
  sta MMC2_CHRBANK0_FD
  sta MMC2_CHRBANK0_FE
  sta MMC2_CHRBANK1_FD
  sta MMC2_CHRBANK1_FE
  eor #$3F

  ; Set latches to "wrong" tile
  ldy #0
  sta (wndreg_lo),y
  ldy #$0F
  sty PPUADDR
  lda mmc2_wrong_low,x
  sta PPUADDR
  bit PPUDATA
  ldy #$1F
  sty PPUADDR
  sta PPUADDR
  bit PPUDATA

  ; Read back the "wrong" tile
  ldy cur_tile_hi
  sty PPUADDR
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  lda cur_bank
  cmp PPUDATA
  beq fail1

  ; Set latch to "right" tile
  tya
  ora #$0F
  sta PPUADDR
  lda mmc2_wrong_low,x
  eor #$30
  sta PPUADDR
  bit PPUDATA

  ; Read back the "right" tile
  sty PPUADDR
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  lda cur_bank
  cmp PPUDATA
  bne fail1
  cmp #0
  beq done
  dec cur_bank
  jmp loop
done:
  rts
fail1:
  lda #1
  cpx #2
  adc #0
  rts
.endproc

mmc2_window_reg:
  .byte >MMC2_CHRBANK0_FD, >MMC2_CHRBANK0_FE
  .byte >MMC2_CHRBANK1_FD, >MMC2_CHRBANK1_FE
mmc2_wrong_low:  .byte $E8, $D8, $E8, $D8
mmc2_base_high:  .byte $01, $01, $11, $11
