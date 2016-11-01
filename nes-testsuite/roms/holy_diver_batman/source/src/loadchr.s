;
; CHR ROM/RAM test for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/nes.h"
.include "src/ram.h"

.segment "ZEROPAGE"
is_chrrom:        .res 1
last_chr_bank:    .res 1
chr_test_result:  .res 1

.segment "CODE"
.proc detect_chrrom
  lda #$00
  jsr driver_load_chr_8k
  ldy #$00
  sty PPUADDR
  sty PPUADDR
  bit PPUDATA
  lda PPUDATA
  eor #$FF
  sty PPUADDR
  sty PPUADDR
  sta PPUDATA
  sty PPUADDR
  sty PPUADDR
  bit PPUDATA
  eor PPUDATA
  sta is_chrrom
  jsr get_chr_size
  lda is_chrrom
  beq do_chrramtest

  ; So we have CHR ROM and we know its size.
  ; Look for small font in CHR ROM
  ldy #$00
  jsr driver_load_chr_8k
  ldy #$00  ; Y = base in vram
  jsr verify_small_font
  beq small_font_ok
  lda #VBLANK_NMI
  sta PPUCTRL
:
  lda #MORSE_F
  jsr morsebeep
  lda #MORSE_O
  jsr morsebeep
  lda #MORSE_N
  jsr morsebeep
  ldy #2*TWOCELLS
  jsr wait_y_frames
  jmp :-
small_font_ok:

cur_bank = $02
  ; Try reading back bank tags from CHR ROM
  lda last_chr_bank
  sta cur_bank
loop:
  lda cur_bank
  jsr driver_load_chr_8k
  jsr read_chr_bank_tags
  cpx #0
  bne chrrom_problem
  lsr a
  lsr a
  lsr a
  cmp cur_bank
  bne chrrom_problem
  dec cur_bank
  bpl loop
  lda #0
  beq have_chrrom_result

chrrom_problem:
  lda #$FF
have_chrrom_result:
  sta chr_test_result
  rts

do_chrramtest:
  ; Try reading and writing a pattern
  jsr full_vram_test
  sta chr_test_result
  lda #$30
  sta $400C
  jsr load_small_font
  jmp get_chr_size  ; put the bank tags back
.endproc

.proc load_small_font
fontsrc = 0
fontdstlo = 2
  lda #VBLANK_NMI
  ldy #$00
  sty PPUADDR
  sty PPUADDR
  ldx #32
  tya
:
  sta PPUDATA
  iny
  bne :-
  dex
  bne :-
  sty PPUADDR
  sty PPUADDR

  lda #VBLANK_NMI|VRAM_DOWN
  sta PPUCTRL
  lda #<romfont
  sta fontsrc
  lda #>romfont
  sta fontsrc+1
  tya
streamloop:
  ; at this point, A is the low order bit of the destination address
  ldy #$00
  sty PPUADDR
  sta fontdstlo
  sta PPUADDR
streambyteloop:
  lda (fontsrc),y
  sta PPUDATA
  iny
  cpy #32
  bcc streambyteloop
  tya
  clc
  adc fontsrc
  sta fontsrc
  bcc :+
  inc fontsrc+1
:
  lda fontdstlo
  clc
  adc #16
  cmp #32
  bcc streamloop
  sbc #31
  cmp #5
  bcc streamloop

  lda #VBLANK_NMI  ; reset writing direction
  sta PPUCTRL
  rts
.endproc

;;
; Verifies that the small font is loaded into CHR ROM or RAM.
; @return A=0, Z=true if same, A=$FF, Z=false if different
.proc verify_small_font
fontsrc = 0
fontdstlo = 2
  lda #VRAM_DOWN
  sta PPUCTRL
  lda #<romfont
  sta fontsrc
  lda #>romfont
  sta fontsrc+1
  tya
streamloop:
  ; at this point, A is the low order bit of the destination address
  ldy #$00
  sty PPUADDR
  sta fontdstlo
  sta PPUADDR
  bit PPUDATA
streambyteloop:
  lda (fontsrc),y
  cmp PPUDATA
  bne fail
  iny
  cpy #32
  bcc streambyteloop
  tya
  clc
  adc fontsrc
  sta fontsrc
  bcc :+
  inc fontsrc+1
:
  lda fontdstlo
  clc
  adc #16
  cmp #32
  bcc streamloop
  sbc #31
  cmp #5
  bcc streamloop

  lda #VBLANK_NMI  ; reset writing direction
  sta PPUCTRL
  asl a  ; end with Z flag on
  rts
fail:
  lda #$FF  ; end with Z flag off
  rts
.endproc

;;
; Writes bank tags at CHR $01FC, $05FC, $09FC, ..., $1DFC OR'd with A
.proc write_chr_bank_tags
banktagbase = $00
  sta banktagbase
  ldx #7
loop:
  txa
  asl a
  sec
  rol a
  sta PPUADDR
  lda #$FC
  sta PPUADDR
  txa
  ora banktagbase
  sta PPUDATA
  dex
  bpl loop
  rts
.endproc

;;
; Reads the bank tag at CHR $01FC and compares it to those at
; $05FC, $09FC, ..., $1DFC
; @return A: last read bank tag
; X: 0 for match or >0 for mismatch
.proc read_chr_bank_tags
banktagbase = $00
expected_value = $01
  lda #$01
  sta PPUADDR
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  lda PPUDATA
  sta banktagbase
  and #$07
  bne bail
  ldx #7
loop:
  txa
  asl a
  sec
  rol a
  sta PPUADDR
  lda #$FC
  sta PPUADDR
  bit PPUDATA
  txa
  ora banktagbase
  sta expected_value
  lda PPUDATA
  cmp expected_value
  bne bail
  dex
  bne loop
bail:
  rts
.endproc

;;
; Assuming either CHR ROM or CHR RAM that has been filled with
; bank tags.
.proc get_chr_size
  lda is_chrrom
  bne skip_writing

  lda #$1F
  sta 8
:
  lda 8
  jsr driver_load_chr_8k
  lda 8
  asl a
  asl a
  asl a
  jsr write_chr_bank_tags
  dec 8
  bpl :-
skip_writing:

  lda #$1F
  jsr driver_load_chr_8k
  jsr read_chr_bank_tags
  cpx #0
  beq read_back_success
  lda #VBLANK_NMI
  sta PPUCTRL
:
  lda #MORSE_C
  jsr morsebeep
  lda #MORSE_B
  jsr morsebeep
  lda #MORSE_T
  jsr morsebeep
  ldy #TWOCELLS*2
  jsr wait_y_frames
  jmp :-

read_back_success:
  lsr a
  lsr a
  lsr a
  sta last_chr_bank
  rts
.endproc

;;
; Fills all of PRG RAM with pseudorandom patterns and makes sure
; they read back correctly.
.proc full_vram_test
ptrlo = 0
ptrhi = 1
ptrbank = 2
cur_round = 3

  lda #$08
  sta $4015
  lda #$30
  sta $400C
  ldy #$8A
  sty $400E
  sty $400F
  ldy #7
roundloop:
  sty cur_round
  
  ; Indicate progress
  tya
  asl a
  eor #$3E
  sta $400C
  
  ; Fill RAM with this bank's fill pattern
  lda last_chr_bank
  sta ptrbank
  lda round_fillvals,y
fill_bankloop:
  pha
  lda ptrbank
  jsr driver_load_chr_8k
  pla
  ldy #$60
  sty ptrhi
  ldy #$00
  sty PPUADDR
  sty PPUADDR
fill_byteloop:
  sta PPUDATA
  crc8_update_0
  iny
  bne fill_byteloop
  inc ptrhi
  bpl fill_byteloop
  dec ptrbank
  bpl fill_bankloop

  ; Indicate progress
  ldy cur_round
  tya
  asl a
  eor #$3F
  sta $400C
  
  ; And verify that this pattern was written successfully
  lda last_chr_bank
  sta ptrbank
  lda round_fillvals,y
read_bankloop:
  pha
  lda ptrbank
  jsr driver_load_chr_8k
  pla
  ldy #$60
  sty ptrhi
  ldy #$00
  sty PPUADDR
  sty PPUADDR
  bit PPUDATA
read_byteloop:
  cmp PPUDATA
  bne test_failed
  crc8_update_0
  iny
  bne read_byteloop
  inc ptrhi
  bpl read_byteloop
  dec ptrbank
  bpl read_bankloop
  
  ldy cur_round
  dey
  bpl roundloop
  lda #0
  rts
test_failed:
  lda #$FF
  rts
.endproc



;;
; @param A high byte of string
; @param Y low byte of string
.proc smallchr_puts_multiline
srcLo = $00
srcHi = $01
dstLo = $02
dstHi = $03

  sta srcHi
  ldx #0
  stx srcLo  ; keep the low byte in Y

newline:
  lda dstHi
  sta PPUADDR
  lda dstLo
  sta PPUADDR
  clc
  adc #32
  sta dstLo
  bcc :+
  inc dstHi
:
  ldx #0
charloop:
  lda (srcLo),y
  beq done
  iny
  bne :+
  inc srcHi
:
  cmp #LF
  beq newline
  and #$3F
  sta PPUDATA
  bpl charloop
done:
  rts
.endproc

.segment "RODATA"
romfont:
  .incbin "obj/nes/font8x5.bin"
.segment "CHR"
  .incbin "obj/nes/font8x5.chr"

