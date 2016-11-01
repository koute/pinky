;
; Backup memory ($6000-$7FFF) test for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/nes.h"
.include "src/ram.h"

CUR_WRAM_BANK = $60FF
savedata = $6100

.segment "ZEROPAGE"
has_savedata:   .res 1
has_wram:       .res 1
last_wram_bank: .res 1
wram_test_result: .res 1

.segment "CODE"
.proc store_savedata
  ldx #7
loop:
  lda savedata_sig,x
  sta savedata,x
  dex
  bpl loop
  rts
.endproc

.proc verify_savedata
  ldx #7
loop:
  lda savedata_sig,x
  eor savedata,x
  bne done
  dex
  bpl loop
  lda #0
done:
  rts
.endproc

.proc wram_test
  lda #0
  jsr driver_load_prg_ram_8k
  jsr verify_savedata
  beq :+
  lda #1
:
  eor #$01
  sta has_savedata

  ; First sanity check that WRAM can hold data for even 4 cycles
  lda #$AA
  sta CUR_WRAM_BANK
  eor CUR_WRAM_BANK
  bne no_wram
  lda #$55
  sta CUR_WRAM_BANK
  eor CUR_WRAM_BANK
  beq yes_wram
no_wram:
  lda #0
  sta has_wram
  rts
yes_wram:
  lda #$FF
  sta has_wram

  ; Spray bank tags to see which ones get mirrored
  lda #$0F
  sta last_wram_bank
load_prg_bank_tags:
  lda last_wram_bank
  jsr driver_load_prg_ram_8k
  lda last_wram_bank
  sta CUR_WRAM_BANK
  dec last_wram_bank
  bpl load_prg_bank_tags

  ; Assuming no funny business (such as SOROM using banks 0 and 2
  ; only), the last bank number should be mirrored to the last unique
  ; bank, and the bank number written to that bank should be 1 less
  ; than the total number of unique banks.
  lda #$0F
  jsr driver_load_prg_ram_8k
  lda CUR_WRAM_BANK
  sta last_wram_bank

  ; Now test every single byte of PRG RAM.  Fortunately, none of the
  ; PRG RAM bankswitch routines modify memory.
  jsr full_wram_test
  sta wram_test_result
  lda #$30
  sta $400C

  ; Finally, place the signature back in PRG RAM for
  ; battery detection.
  lda #0
  jsr driver_load_prg_ram_8k
  jmp store_savedata
.endproc


;;
; Fills all of PRG RAM with pseudorandom patterns and makes sure
; they read back correctly.
; @return 0 for success, nonzero for failure
.proc full_wram_test
ptrlo = 0
ptrhi = 1
ptrbank = 2
cur_round = 3

  lda #$08
  sta $4015
  lda #$30
  sta $400C
  ldy #$88
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
  lda last_wram_bank
  sta ptrbank
  lda round_fillvals,y
fill_bankloop:
  pha
  lda ptrbank
  jsr driver_load_prg_ram_8k
  pla
  ldy #$60
  sty ptrhi
  ldy #$00
  sty ptrlo
fill_byteloop:
  sta (ptrlo),y
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
  lda last_wram_bank
  sta ptrbank
  lda round_fillvals,y
read_bankloop:
  pha
  lda ptrbank
  jsr driver_load_prg_ram_8k
  pla
  ldy #$60
  sty ptrhi
  ldy #$00
  sty ptrlo
read_byteloop:
  cmp (ptrlo),y
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

.segment "RODATA"
savedata_sig:
  .byte "SAVEDATA"
round_fillvals:
  .repeat 8, I
    .byte 1 << I
  .endrepeat
