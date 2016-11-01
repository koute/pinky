;
; Board letter lookup for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/ram.h"

;;
; Calculates the letter associated with a board name.
; @return A: first letter (or >=$80 if not found)
;         Y: second letter (or >=$80 if 1 letter)
.proc get_board_name
srclo = 0
srchi = 1
numleft = 2
expected_wram = 3
  lda has_wram
  beq have_expected_wram
  lda #1
  ldy has_savedata
  cpy #1   ; No save: 0 00000001  With save: 1 00000001
  ror a    ; No save: 1 00000000  With save: 1 10000000
  adc last_wram_bank  ; A = number of WRAM banks + 128 if battery
have_expected_wram:
  sta expected_wram

  lda #<letterdata
  sta srclo
  lda #>letterdata
  sta srchi
  lda #letterdata_numboards
  sta numleft
loop:
  ldy #0
  lda (srclo),y   ; [0]: mapper
  cmp cur_mapper
  bne skip
  iny
  lda last_prg_bank
  cmp (srclo),y   ; [1]: minimum last PRG ROM bank (4K)
  bcc skip
  iny
  lda (srclo),y   ; [2]: maximum last PRG ROM bank
  cmp last_prg_bank
  bcc skip
  iny
  lda last_chr_bank
  cmp (srclo),y   ; [3]: minimum last CHR bank (8K)
  bcc skip
  iny
  lda (srclo),y   ; [4]: maximum last CHR bank
  cmp last_chr_bank
  bcc skip
  iny
  lda expected_wram
  cmp (srclo),y   ; [5]: number of WRAM banks, plus 128 for battery
  beq found
  and #$7F        ; TKROM is the only board that requires a battery;
  cmp (srclo),y   ; other boards with WRAM can be used with or without
  bne skip
found:
  iny
  lda (srclo),y
  pha
  iny
  lda (srclo),y
  tay
  pla
  rts
skip:
  lda srclo
  clc
  adc #8
  sta srclo
  bcc :+
  inc srchi
:
  dec numleft
  bne loop
done:
  lda #$80
  rts
.endproc

.macro board mapper, min_prg, max_prg, min_chr, max_chr, wramsize, name
  .byte mapper
  .byte (min_prg / 4) - 1, (max_prg / 4) - 1
  .byte (min_chr / 8) - 1, (max_chr / 8) - 1
  .byte (wramsize) / 8
  .byte .strat(name, 0) & $3F
  .if .strlen(name) < 2
    .byte $80
  .else
    .byte .strat(name, 1) & $3F
  .endif
.endmacro

.segment "RODATA"
letterdata:
;       MAP PRGSIZE CHRSIZE  WR NAME
  board  66, 32, 32,  8,  8,  0, "N"
  board  66, 32, 32, 16, 32,  0, "CN"
  board  66, 64, 64,  8, 16,  0, "MH"
  board  66, 64,128,  8, 32,  0, "GN"
  board   2, 64,128,  8,  8,  0, "UN"
  board   2,256,256,  8,  8,  0, "UO"
  board 180, 64,128,  8,  8,  0, "UN"
  board 180,256,256,  8,  8,  0, "UO"
  board   7, 32,128,  8,  8,  0, "AN"
  board   7,256,256,  8,  8,  0, "AO"
  board  34, 64,128,  8,  8,  0, "BN"
  board   1, 64, 64, 16, 64,  8, "SA"
  board   1, 64, 64, 16, 64,  0, "SB"
  board   1, 64, 64,128,128,  0, "SC"
  board   1, 32, 32, 16, 64,  0, "SE"
  board   1,128,256, 16, 64,  0, "SF"
  board   1,128,256,  8,  8,  0, "SG"
  board   1, 32, 32,128,128,  0, "SH"
  board   1, 32, 32, 16, 64,  8, "SI"
  board   1,128,256, 16, 64,  8, "SJ"
  board   1,128,256,128,128,  8, "SK"
  board   1,128,256,128,128,  0, "SL"
  board   1,128,256,  8,  8,  8, "SN"
  board   1,512,512,  8,  8,  8, "SU"
  board   1,128,512,  8,  8, 32, "SX"
  board   4, 64, 64, 16, 64,  0, "TB"
  board   4, 32, 32, 16, 64,  0, "TE"
  board   4,128,512, 16, 64,  0, "TF"
  board   4,128,512,  8,  8,  0, "TG"
  board   4,128,512,128,256,  8+1024, "TK"
  board   4,128,512,128,256,  0, "TL"
  board   4,128,512,  8,  8,  8, "TN"
  board   4,128,512,128,256,  8, "TS"
  board  69,256,256,128,128,  0, "JL"
  board  69,128,128,256,256,  8, "JS"
  board 118,128,256,128,128,  8, "TK"
  board 118,128,512,128,128,  0, "TL"
letterdata_numboards = (* - letterdata) / 8
