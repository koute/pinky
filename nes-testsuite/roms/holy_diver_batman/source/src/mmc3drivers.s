;
; Test drivers for MMC3-class mappers for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/nes.h"
.include "src/ram.h"

; MMC3 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

.segment "DRIVER_MMC3"
  jmp mmc3_set_wram_8k
  jmp mmc3_set_chr_8k
.proc mmc3_test_mapper
  ; Read all PRG ROM bank tags through each PRG ROM window
  lda #$00
  sta driver_prg_result
  ldx #3
prgwinloop:
  jsr mmc3_test_one_prg_window
  asl a
  rol driver_prg_result
  dex
  bpl prgwinloop

  ; Verify PRG RAM protection
  jsr mmc3_test_wram_protection
  ora driver_prg_result
  sta driver_prg_result

  ; Test that IRQ line is connected
  jsr mmc3_test_irq
  sta driver_chr_result

  ; Read all CHR bank tags through each single CHR window
  ldx #15
chrwinloop:
  jsr mmc3_test_one_chr_window
  cpx #8
  bcc :+
  asl a
:
  ora driver_chr_result
  sta driver_chr_result
  dex
  bpl chrwinloop
  rts
.endproc
.proc mmc3_set_wram_8k
  and #%00111111
  ora #MMC3_WRAM_RW
  sta MMC3_WRAM_ENABLE
  rts
.endproc
.proc mmc3_set_chr_8k
  asl a
  asl a
  asl a
  ldy #0
  sty MMC3_SELECT
  sta MMC3_DATA
  ora #$03
  clc
loop:
  iny
  sty MMC3_SELECT
  sta MMC3_DATA
  adc #1
  cpy #5
  bcc loop
  rts
.endproc

;;
; Tests all banks in one window. 
; @param X window id (0-3)
; @return A=$00 for success, $FF for failure; X unchanged
.proc mmc3_test_one_prg_window
addrlo = $00
addrhi = $01
banksleft = $02
  ; Swap the wrong bank into the other window
  lda mmc3_prgwindow_select,x
  eor #$01
  sta MMC3_SELECT
  ldy #$FE
  sty MMC3_DATA
  eor #$01
  sta MMC3_SELECT
  
  ; Put a pointer to this window
  lda #<CUR_BANK
  sta addrlo
  lda mmc3_prgwindow_addrbase,x
  sta addrhi
  lda last_prg_bank
  sta banksleft
  ldy #0
loop:
  ; Copy bit 0 of banksleft into bit 4 of addrhi
  lda banksleft
  lsr a
  sta MMC3_DATA
  lda addrhi
  and #%11101111
  bcc :+
  ora #$10
:
  sta addrhi
  lda (addrlo),y
  cmp banksleft
  bne test_failed
  cmp #0
  bne :+
  rts
:
  dec banksleft
  jmp loop
test_failed:
  lda #$FF
  rts
.endproc

mmc3_prgwindow_select:
  .byte $06, $07, $46, $47
mmc3_prgwindow_addrbase:
  .byte >(CUR_BANK-$7000), >(CUR_BANK-$5000)
  .byte >(CUR_BANK-$3000), >(CUR_BANK-$5000)

.proc mmc3_test_one_chr_window
ppuaddrhi = $00
chr_and = $01
chr_or = $02

  txa
  and #$07
  tay
  ; A, Y: window id wrapped to $00-$07
  
  ; Calculate address of CHR bank tag within the window
  asl a
  sec
  rol a
  cpx #8
  bcc :+
  eor #$10  ; 8: Flipped
:
  sta ppuaddrhi

  ; Calculate the values to AND and OR with the written bank number
  ; to get the expected bank tag read through the window
  lda #$7F
  cpy #4
  rol a
  sta chr_and  ; FE FE FE FE FF FF FF FF
  lda mmc3_chr_or,y
  sta chr_or

  ; Select the desired window
  lda mmc3_chr_select,y
  cpx #8
  ror a
  sta MMC3_SELECT

  lda last_chr_bank
  asl a
  asl a
  asl a
  ora #$07
  tay
loop:
  sty MMC3_DATA
  lda ppuaddrhi
  sta PPUADDR  ; 01FC, 05FC, 09FC, ..., 1DFC
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  tya
  and chr_and
  ora chr_or
  cmp PPUDATA
  bne fail
  cpy #0
  beq success
  dey
  jmp loop

success:
  lda #0
  rts
fail:
  lda #MAPTEST_CHR
  rts
.endproc

; values written to $8000, shifted left by 1
mmc3_chr_select: .byte 0, 0, 2, 2, 4, 6, 8, 10
; values OR'd with CHR bank number to produce expected bank tag
mmc3_chr_or:     .byte 0, 1, 0, 1, 0, 0, 0, 0


.proc mmc3_test_wram_protection
  lda has_wram
  beq skip_wram_protection_test

  ; make sure writes at least take 
  ldy #MMC3_WRAM_RW
  sty MMC3_WRAM_ENABLE
  ldx #$6B  ; X is the "bad" value
  lda #$B6  ; A is the "good" value
  sta $6000
  cmp $6000
  bne protproblem

  ; make sure writes don't take if set to read-only
  ldy #MMC3_WRAM_RO
  sty MMC3_WRAM_ENABLE
  stx $6000
  cmp $6000
  bne roproblem

  ; make sure it can't be read from open bus
  ldy #MMC3_WRAM_OFF
  sty MMC3_WRAM_ENABLE
  cmp $6000
  beq protproblem

  ; test for $6000/$E000 glitch (the bug that caused map corruption
  ; in Crystalis and M.C. Kids on old versions of PowerPak MMC3)
  ldy #MMC3_WRAM_RW
  sty MMC3_WRAM_ENABLE
  stx MMC3_IRQ_DISABLE
  cmp $6000
  bne e000problem

skip_wram_protection_test:
  lda #0
  rts
protproblem:
  lda #MAPTEST_WRAMEN
  rts
roproblem:
  lda #MAPTEST_WRAMRO
  rts
e000problem:
  lda #MAPTEST_WRAMEN2
  rts
.endproc

.proc mmc3_test_irq
  lda nmis
:
  cmp nmis
  beq :-
  lda #JMP_OPCODE
  sta irq_handler
  lda #<mmc3_irq_handler
  sta irq_handler+1
  lda #>mmc3_irq_handler
  sta irq_handler+2
  lda #BG_ON
  sta PPUMASK
  lda #VBLANK_NMI|BG_0000|OBJ_1000
  sta PPUCTRL
  lda #0
  sta PPUSCROLL
  sta PPUSCROLL
  sta irqs
  lda #64
  sta MMC3_IRQ_PERIOD
  sta MMC3_IRQ_RELOAD
  sta MMC3_IRQ_ENABLE
  
  ; Now wait a frame for the IRQs to come in
  cli
  lda nmis
:
  cmp nmis
  beq :-
  sei
  lda #0
  sta PPUMASK
  lda irqs
  eor #3
  beq :+
  lda #MAPTEST_IRQ
:
  rts
.endproc
.proc mmc3_irq_handler
  inc irqs
  sta MMC3_IRQ_DISABLE
  sta MMC3_IRQ_ENABLE
  rti
.endproc

; FME-7 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; FME7_CHRBANK+0-7: Swap 1K CHR into $0000, $0400, $0800, ..., $1C00
; FME7_PRGBANK+0-3: Swap 8K PRG into $6000, $8000, $A000, $C000
; $6000 is special: $00 is ROM, $80 is open bus, $C0 is RAM
; FME7_IRQCTRL:
;   FME7_IRQ_COUNT means run counter
;   FME7_IRQ_ENABLE means generate IRQs (clear then set to ack)
;FME7_IRQ_LOW and FME7_IRQ_HIGH set the counter
; when counter becomes $FFFF while counting, assert /IRQ until ack

.segment "DRIVER_FME7"
  jmp fme7_set_prg_8k
  jmp fme7_set_chr_8k
.proc fme7_test_mapper
  ; Read all PRG ROM bank tags through each PRG ROM window
  lda #$00
  sta driver_prg_result
  ldx #3
prgwinloop:
  jsr fme7_test_one_prg_window
  asl a
  rol driver_prg_result
  dex
  bpl prgwinloop

  ; Verify PRG RAM protection
  jsr fme7_test_wram_protection
  ora driver_prg_result
  sta driver_prg_result

.if 1
  ; Test that IRQ line is connected
  jsr fme7_test_irq
.else
  lda #0
.endif
  sta driver_chr_result

  ; Read all CHR bank tags through each single CHR window
  ldx #7
chrwinloop:
  jsr fme7_test_one_chr_window
  ora driver_chr_result
  sta driver_chr_result
  dex
  bpl chrwinloop
  rts
.endproc
.proc fme7_set_prg_8k
  and #$1F
  ora #FME7_PRGBANK_RAM
  ldy #FME7_PRGBANK+0
  sty FME7_SELECT
  sta FME7_DATA
  rts
.endproc
.proc fme7_set_chr_8k
  asl a
  asl a
  asl a
  clc
  ldy #FME7_CHRBANK
loop:
  sty FME7_SELECT
  sta FME7_DATA
  adc #1
  iny
  cpy #FME7_CHRBANK+8
  bcc loop
  rts
.endproc

;;
; Tests all banks in one window. 
; @param X window id (0-3)
; @return A=$00 for success, $FF for failure; X unchanged
.proc fme7_test_one_prg_window
addrlo = $00
addrhi = $01
banksleft = $02

  ; First swap the default bank into all windows
  ldy #$08
  lda #$3B
  clc
clrbanksloop:
  sty FME7_SELECT
  sta FME7_DATA
  iny
  adc #1
  cpy #12
  bcc clrbanksloop
  
  txa
  ora #FME7_PRGBANK
  sta FME7_SELECT

  ; Get a pointer to this window
  lda #<CUR_BANK
  sta addrlo
  lda fme7_prgwindow_addrbase,x
  sta addrhi
  lda last_prg_bank
  sta banksleft
  ldy #0
loop:
  ; Copy bit 0 of banksleft into bit 4 of addrhi
  lda banksleft
  lsr a
  sta FME7_DATA
  lda addrhi
  and #%11101111
  bcc :+
  ora #$10
:
  sta addrhi
  lda (addrlo),y
  cmp banksleft
  bne test_failed
  cmp #0
  bne :+
  rts
:
  dec banksleft
  jmp loop
test_failed:
  lda #$FF
  rts
.endproc

fme7_prgwindow_addrbase:
  .byte >(CUR_BANK-$9000), >(CUR_BANK-$7000)
  .byte >(CUR_BANK-$5000), >(CUR_BANK-$3000)

.proc fme7_test_one_chr_window
ppuaddrhi = $00

  ; Fortunately FME-7 is more orthogonal than MMC3.  It doesn't have
  ; 2K windows or exchanging $0000 with $1000.
  ; Calculate address of CHR bank tag within the window
  txa
  asl a
  sec
  rol a
  sta ppuaddrhi

  ; Select the desired window
  stx FME7_SELECT

  lda last_chr_bank
  asl a
  asl a
  asl a
  ora #$07
  tay
loop:
  sty FME7_DATA
  lda ppuaddrhi
  sta PPUADDR  ; 01FC, 05FC, 09FC, ..., 1DFC
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  cpy PPUDATA
  bne fail
  cpy #0
  beq success
  dey
  jmp loop

success:
  lda #0
  rts
fail:
  lda #MAPTEST_CHR
  rts
.endproc

; The IS_LAST_BANK location mirrored into the WRAM window
WRAM_LOC = IS_LAST_BANK - $8000
.proc fme7_test_wram_protection
  lda has_wram
  beq skip_test

  ; Write to last bank of RAM
  lda #FME7_PRGBANK+0
  sta FME7_SELECT
  lda #FME7_PRGBANK_RAM|$3F
  sta FME7_DATA
  lda #2
  sta WRAM_LOC
  cmp WRAM_LOC
  bne protproblem2

  ; Read last bank of ROM
  lda #FME7_PRGBANK_ROM|$3F
  sta FME7_DATA
  lda WRAM_LOC
  cmp IS_LAST_BANK
  bne roproblem

  ; Read open bus
  lda #FME7_PRGBANK_OFF|$3F
  sta FME7_DATA
  lda WRAM_LOC  ; open_bus should be $7F
  cmp #3        ; 0: most ROM banks; 1: last ROM bank; 2: last RAM bank
  bcc protproblem

skip_test:
  lda #0
have_result:
  ldy #FME7_PRGBANK_RAM|$3F
  sty FME7_DATA
  rts
protproblem:
  lda #MAPTEST_WRAMEN
  bne have_result
roproblem:
  lda #MAPTEST_WRAMRO
  bne have_result
protproblem2:
  lda #MAPTEST_WRAMEN2
  bne have_result
.endproc

.proc fme7_test_irq
  ; Set the IRQ vector
  lda #JMP_OPCODE
  sta irq_handler
  lda #<fme7_irq_handler
  sta irq_handler+1
  lda #>fme7_irq_handler
  sta irq_handler+2

  ; Schedule an IRQ 256 cycles from now
  ldx #$0D
  stx FME7_SELECT
  ldy #0
  sty FME7_DATA  ; $00: don't count
  sty PPUCTRL    ; disable nmi because we're concentrating on irq
  inx
  stx FME7_SELECT
  sty FME7_DATA  ; $00: low 8 bits of count
  inx
  stx FME7_SELECT
  ldx #1         ; $01: high 8 bits of count
  stx FME7_DATA
  ldx #$0D
  stx FME7_SELECT
  ldx #$81
  lda irqs
  cli
  stx FME7_DATA  ; now start counting and IRQing
  ldy #-27
waitfast:        ; 10 cycles per loop
  iny
  beq waitfail
  cmp irqs
  beq waitfast
  cpy #-2
  bcs waitfast_ok
waitfail:
  lda #MAPTEST_IRQ
  sei
  rts
waitfast_ok:

  ; Now wait for the next IRQ, 65536 cycles later
  ldx #-53
  lda irqs
waitslow:        ; 1289 cycles per loop
  iny
  bne waitslow
  inx
  beq waitfail
  cmp irqs
  beq waitslow
  cpx #-4
  bcc waitfail

  lda #0
  sei
  rts
.endproc

;;
; If you're doing any bank switching in the main thread while IRQs
; are turned on, you need to save the value that you're writing to
; FME7_SELECT before writing it in the main thread and restore it
; in the IRQ handler.
; MMC3 is a lot more convenient in this respect: the IRQ portion
; is essentially a separate device from the bank switching portion.
.proc fme7_irq_handler
  inc irqs
  pha
  lda #$0D
  sta FME7_SELECT
  lda #$80  ; disable IRQ
  sta FME7_DATA
  lda #$81  ; reenable IRQ
  sta FME7_DATA
  ; Here you'd restore the old FME7_SELECT value.
  pla
  rti
.endproc


