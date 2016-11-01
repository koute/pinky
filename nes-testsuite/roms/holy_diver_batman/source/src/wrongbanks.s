;
; First stage bootloader for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;

; If the CPU powers up with any bank in $F000-$FFFF other than the last,
; beep out "WB"

.include "src/nes.h"
.include "src/ram.h"
.importzp nmis
.import reset

; TWOCELLS is the number of frames in a dot and the space after it.
; Because a standard word is 50 cells (25*TWOCELLS), and the NTSC NES
; runs 60.1 fps, the words per minute value is 144.24/TWOCELLS.
TWOCELLS = 7

; Morse letters with 0=dot, 1=dash, and 1000...=terminate

; Several emulators don't load the last bank at power-on for AOROM,
; BNROM, GNROM, and UNROM Crazy.  Instead they load the first bank.
; Work around these by putting a special reset stub at $B000 in
; the first bank.  ($8000-$BFFF is in both the first and last banks.)
; May need more testing to determine whether the stub needs to be
; placed in all 16K banks.
.segment "GNROMSTUB"
.proc gnromstub
  sei
  lda #$FF
  sta CONSTANT_FF
  lda IS_LAST_BANK
  beq isnt_last_bank
  jmp reset
isnt_last_bank:
  jmp wrongbank_start
.endproc

.segment "WRONGBANK"
wrongbank_start:

.proc wrongbank_reset
  sei
  ldx #$00
  stx PPUCTRL
  stx PPUMASK
  ldx #2
:
  bit $2002
  bpl :-
  dex
  bpl :-
  txs

  lda #$04
  sta $4015
  lda #VBLANK_NMI
  sta PPUCTRL
loop:
  lda #MORSE_W
  jsr morsebeep
  lda #MORSE_B
  jsr morsebeep
  ; interword space is 4 cells longer than intercharacter
  ldy #2*TWOCELLS
  jsr wait_y_frames
  jmp loop
.endproc

.proc morsebeep
morse_bits = 0
  asl a
loop:
  pha

  ; A dot is 1 cell (TWOCELLS/2) on and 1 cell off
  ; A dash is 3 cells (3*TWOCELLS/2) on and 1 cell off

  ; carry is 0: dot, 1: dash
  ldy #(TWOCELLS + 1)/2
  bcc is_dot
  ldy #(3*TWOCELLS + 1)/2
is_dot:

  ; Turn on tone at 1000 Hz
  lda #$B0
  sta $4008
  lda #55  ; 1000 Hz
  sta $400a
  lda #0
  sta $400b
  jsr wait_y_frames

  ; Turn off tone 
  sty $4008
  sty $400b
  lda #$80
  sta $4017
  ldy #TWOCELLS/2
  jsr wait_y_frames

  ; Get next bit
  pla
  asl a
  bne loop

  ; Intercharacter space is 3 cells, of which we've waited one
  ldy #TWOCELLS
  ; fall through
.endproc
;;
; Waits for nmis to be changed Y times, then loads Y with 0 and
; leaves X unchanged.
.proc wait_y_frames
  lda nmis
:
  cmp nmis
  beq :-
  dey
  bne wait_y_frames
  rts
.endproc
.proc wrongbank_nmi
  inc nmis
  rti
.endproc

  .res wrongbank_start + $76 - *
CONSTANT_00:  .byt $00
CONSTANT_FF:  .byt $FF
CUR_BANK:     .byt $07
IS_LAST_BANK: .byt $01
  .addr wrongbank_nmi, reset, irq_handler

