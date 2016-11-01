;
; Holy Diver Batman: NES cartridge board test
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;

.include "src/nes.h"
.include "src/ram.h"

.exportzp cur_keys, new_keys
.import __LOWCODE_LOAD__, __LOWCODE_RUN__, __LOWCODE_SIZE__

; Zero page top:
nmis = $FF
irqs = $FE
irq_handler = $F0

OAM = $0200

.segment "ZEROPAGE"

cur_keys:       .res 2
new_keys:       .res 2
oam_used:       .res 1
cur_mapper:     .res 1
txt_y:          .res 1
last_prg_bank:  .res 1

INES_MAPPER = MAPPER_FME7
INES_MIRRH = $00
INES_MIRRV = $01
INES_MIRR4 = $08
INES_FLAGS6 = INES_MIRRV
NES2_SIGNATURE = $08

; FCEUX treats mapper 78 with unknown CRCs as submapper 1 (single
; screen mirroring).  Holy Diver requires submapper 3 (switchable
; H/V mirroring).  NESEMU2 by Dead_Body added support for 78.3
; git clone https://github.com/holodnak/nesemu2.git
.if INES_MAPPER = MAPPER_HOLYDIVER
  INES_SUBMAPPER = 3
.else
  INES_SUBMAPPER = 0
.endif

.segment "INESHDR"
  .byt "NES",$1A  ; magic signature
  .byt 2          ; PRG ROM size in 16384 byte units
  .byt 1          ; CHR ROM size in 8192 byte units
  .byt <(INES_MAPPER << 4) | INES_FLAGS6
  .byt (INES_MAPPER & $F0) | NES2_SIGNATURE
  .byt INES_SUBMAPPER << 4

.segment "STARTUP"
.export reset, irq

identity:
  .repeat 256, I
    .byte I
  .endrepeat

; Will need to modify this once we test MMC3 IRQs
.proc irq
  rti
.endproc

.proc reset
  ; The very first thing to do when powering on is to put all sources
  ; of interrupts into a known state.
  sei             ; Disable interrupts
  ldx #$00
  stx PPUCTRL     ; Disable NMI and set VRAM increment to 32
  stx PPUMASK     ; Disable rendering
  stx $4010       ; Disable DMC IRQ
  dex             ; Subtracting 1 from $00 gives $FF, which is a
  txs             ; quick way to set the stack pointer to $01FF
  bit PPUSTATUS   ; Acknowledge stray vblank NMI across reset
  bit SNDCHN      ; Acknowledge DMC IRQ
  lda #$40
  sta P2          ; Disable APU Frame IRQ
  lda #$0F
  sta SNDCHN      ; Disable DMC playback, initialize other channels

vwait1:
  bit PPUSTATUS   ; It takes one full frame for the PPU to become
  bpl vwait1      ; stable.  Wait for the first frame's vblank.
  
  ; Burn 29700 cycles by clearing OAM and the zero page and copying
  ; any code that must run in RAM to RAM.
  ldy #0
  cld
  lda #<__LOWCODE_LOAD__
  sta 0
  lda #>__LOWCODE_LOAD__
  sta 1
  lda #<__LOWCODE_RUN__
  sta 2
  lda #>__LOWCODE_RUN__
  sta 3
  ldx #<-__LOWCODE_SIZE__
  lda #>-__LOWCODE_SIZE__
  sta 4

copy_lowcode_loop:
  lda (0),y
  sta (2),y
  iny
  bne :+
  inc 1
  inc 3
:
  inx
  bne copy_lowcode_loop
  inc 4
  bne copy_lowcode_loop
  ; at end, X = 0

  txa
clear_zp:
  sta $00,x
  inx
  bne clear_zp
  
vwait2:
  bit PPUSTATUS  ; After the second vblank, we know the PPU has
  bpl vwait2     ; fully stabilized.
  
  jsr detect_mapper
  sta cur_mapper
  jsr ensure_c0_ff
  jmp reset2
.endproc

.proc ensure_c0_ff
  ; Make sure the entire $C000-$FFFF region is swapped in
  lda CUR_BANK
  sta last_prg_bank
  eor CUR_BANK - $1000
  cmp #1
  bne fail16k
  lda last_prg_bank
  eor CUR_BANK - $2000
  cmp #2
  bne fail16k
  lda last_prg_bank
  eor CUR_BANK - $3000
  cmp #3
  bne fail16k
  lda IS_LAST_BANK
  beq fail16k
  rts
fail16k:
  lda #VBLANK_NMI
  sta PPUCTRL
  lda #MORSE_L
  jsr morsebeep
  lda #MORSE_B
  jsr morsebeep
  ldy #2*TWOCELLS
  jsr wait_y_frames
  jmp fail16k
.endproc

; END OF FIRST BOOT CODE ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; By this time, we're sure that the last 16K is visible.

.segment "CODE"
.proc reset2
  ldx #0
  jsr ppu_clear_oam  ; clear out OAM from X to end and set X to 0

  lda cur_mapper
  jsr load_mapper_driver
  jsr detect_chrrom
  jsr wram_test
  lda #$C0
  sta driver_prg_result
  lda #$DE
  sta driver_chr_result
  jsr driver_mapper_test

  ; All tests are complete. Write the results.
  jsr load_main_palette
  lda #$00
  jsr driver_load_chr_8k
  jsr draw_bg
  lda #4
  sta txt_y
  jsr start_line
  lda #VBLANK_NMI
  sta PPUCTRL
  lda cur_mapper
  jsr bcd8bit
  pha
  lda 0
  jsr puthex
  pla
  ora #'0'
  sta PPUDATA
  bit PPUDATA

  ; Write mapper name if known
  jsr get_board_name
  sta 2
  sty 3
  ldx #last_mapper_msg
  lda cur_mapper
mapper_name_search:
  cmp mapper_msgs-3,x
  beq found_mapper_name
  dex
  dex
  dex
  bne mapper_name_search
  beq skip_mapper_name
found_mapper_name:
  ldy mapper_msgs-2,x
  lda mapper_msgs-1,x
  sty 0
  sta 1

  ; If the second character of the board name isn't '*', then the
  ; first two characters don't vary.
  ldy #1
  lda (0),y
  cmp #'*'
  bne skip_letters
  
  ; If the board name wasn't found, then use the generic name with *
  lda 2
  bmi skip_letters
  sta PPUDATA
  lda 3
  bmi :+
  sta PPUDATA
:  
  ldy #2
  bne mapper_name_have_y
skip_letters:
  ldy #0
mapper_name_have_y:
  jsr puts0y
skip_mapper_name:

  lda #0
  sta 1
  lda last_prg_bank
  clc
  adc #1
  rol 1
  asl a
  rol 1
  asl a
  rol 1
  sta 0
  jsr start_line
  jsr write_decimal_in_0
  lda #>k_prg_rom
  ldy #<k_prg_rom
  jsr puts

  ; Write backup size
  jsr start_line
  lda has_wram
  bne print_wram
  lda #>msg_prg_ram
  ldy #<msg_prg_ram
  jsr puts
  lda #>msg_missing
  ldy #<msg_missing
  jsr puts
  jmp skip_print_wram
print_wram:
  lda last_wram_bank
  clc
  adc #1
  asl a
  asl a
  asl a
  sta 0
  lda #0
  sta 1
  jsr write_decimal_in_0
  lda #>k_prg_ram
  ldy #<k_prg_ram
  jsr puts
  lda #>msg_ok
  ldy #<msg_ok
  ldx wram_test_result
  beq :+
  lda #>msg_problem
  ldy #<msg_problem
:
  jsr puts
  lda has_savedata
  beq skip_print_wram
  lda #>msg_battery
  ldy #<msg_battery
  jsr puts
skip_print_wram:

  ; Write CHR size
  lda #0
  sta 1
  lda last_chr_bank
  clc
  adc #1
  rol 1
  asl a
  rol 1
  asl a
  rol 1
  asl a
  rol 1
  sta 0

  jsr start_line
  jsr write_decimal_in_0
  lda #>k_chr
  ldy #<k_chr
  jsr puts
  ldy #'A' & $3F
  lda is_chrrom
  beq :+
  ldy #'O' & $3F
:
  sty PPUDATA
  lda #'M' & $3F
  sta PPUDATA
  lda #>msg_ok
  ldy #<msg_ok
  ldx chr_test_result
  beq :+
  lda #>msg_problem
  ldy #<msg_problem
:
  jsr puts

  jsr start_line
  lda #>msg_detailed
  ldy #<msg_detailed
  jsr puts
  lda driver_prg_result
  jsr puthex
  lda driver_chr_result
  jsr puthex

  jsr start_line
  lda #>msg_flashid
  ldy #<msg_flashid
  jsr puts
  jsr start_line
  bit PPUDATA
  ldx #0
flashid_loop:
  bit PPUDATA
  lda prg_flash_mfrid,x
  jsr puthex
  inx
  cpx #8
  bcc flashid_loop

  ; Now everything is printed.  Turn on the display
  ; and play beep codes.
  ldx #0
  jsr ppu_clear_oam
  jsr vsync
  lda #$0F
  sta SNDCHN

  jsr beepcode_tweet  ; Group 1: Mapper
  lda cur_mapper
  jsr beepcode_byte
  ldy #18
  jsr wait_y_frames

  ; Group 2: PRG ROM size, PRG RAM size, test result
  jsr beepcode_tweet
  lda last_prg_bank
  lsr a
  lsr a
  lsr a
  adc #0
  jsr beepcode_byte
  lda has_wram
  cmp #1
  lda #0
  bcc :+
  adc last_wram_bank
:
  jsr beepcode_byte
  lda wram_test_result
  beq :+
  jsr beepcode_null
:
  lda has_savedata
  beq :+
  jsr beepcode_ding
:
  lda driver_prg_result
  jsr beepcode_byte
  ldy #18
  jsr wait_y_frames

  ; Group 3: CHR size, CHR RAM, test result
  jsr beepcode_tweet
  lda last_chr_bank
  jsr beepcode_byte
  lda is_chrrom
  beq :+
  lda #$07
:
  jsr beepcode_nibble
  lda chr_test_result
  beq :+
  jsr beepcode_null
:
  lda driver_chr_result
  jsr beepcode_byte

forever:

  ; Game logic
  jsr read_pads
  jsr vsync

  jmp forever

; And that's all there is to it.
.endproc

.proc vsync

  ; Wait for a vertical blank and set the scroll
  lda nmis
vw3:
  cmp nmis
  beq vw3
  
  ; Copy the display list from main RAM to the PPU
  lda #0
  sta OAMADDR
  lda #>OAM
  sta OAM_DMA
  
  ; Turn the screen on
  ldx #0
  ldy #0
  lda #VBLANK_NMI|BG_0000|OBJ_1000
  sec
  jmp ppu_screen_on
.endproc

.proc load_main_palette
  ; seek to the start of palette memory ($3F00-$3F1F)
  ldx #$3F
  stx PPUADDR
  ldx #$00
  stx PPUADDR
copypalloop:
  lda initial_palette,x
  sta PPUDATA
  inx
  cpx #32
  bcc copypalloop
  rts
.pushseg
.segment "RODATA"
initial_palette:
  .byt $FF,$38,$38,$20,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF
  .byt $02,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF
.popseg
.endproc

.proc start_line
  lda txt_y
  inc txt_y
  sec
  ror a
  ror a
  ror a
  sta PPUADDR
  ror a
  and #$E0
  ora #$02
  sta PPUADDR
  rts
.endproc

.proc draw_bg
  ; Start by clearing the first nametable
  lda #$80
  sta PPUCTRL
  ldx #$20
  txa
  ldy #$00
  jsr ppu_clear_nt

  ; Draw a floor
  lda #$23
  sta PPUADDR
  lda #$40
  sta PPUADDR
  lda #'_' & $3F
  ldx #32
floorloop1:
  sta PPUDATA
  dex
  bne floorloop1
  
  lda #$21
  sta 3
  lda #$82
  sta 2
  lda #>msg_todo
  ldy #<msg_todo
  jsr smallchr_puts_multiline
  
  rts
.endproc

.proc puthex
  pha
  lsr a
  lsr a
  lsr a
  lsr a
  jsr onedig
  pla
  and #$0F
onedig:
  ora #'0'
  cmp #'0'|$0A
  bcc :+
  adc #'A'-'9'-2-64
:
  sta PPUDATA
  rts
.endproc

.proc puts
  sta 1
  sty 0
.endproc
.proc puts0
  ldy #0
.endproc
.proc puts0y
loop:
  lda (0),y
  beq done
  and #$3F
  sta PPUDATA
  iny
  bne loop
done:
  rts
.endproc

.proc write_decimal_in_0
  jsr bcdConvert
  ldx #4
find1stdigit:
  lda 2,x
  bne got1stdigit
  dex
  bne find1stdigit
got1stdigit:
  lda 2,x
  ora #'0'
  sta PPUDATA
  dex
  bpl got1stdigit
  rts
.endproc

.segment "RODATA"
mapper_msgs:
  .byte MAPPER_HOLYDIVER
  .addr msg_holydiver
  .byte MAPPER_FME7
  .addr msg_fme7
  .byte MAPPER_GNROM
  .addr msg_gnrom
  .byte MAPPER_MMC1
  .addr msg_mmc1
  .byte MAPPER_MMC2
  .addr msg_mmc2
  .byte MAPPER_MMC3
  .addr msg_mmc3
  .byte MAPPER_MMC3_TLSROM
  .addr msg_mmc3_tlsrom
  .byte MAPPER_MMC4
  .addr msg_mmc4
  .byte MAPPER_A53
  .addr msg_a53
  .byte MAPPER_BNROM
  .addr msg_bnrom
  .byte MAPPER_AOROM
  .addr msg_aorom
  .byte MAPPER_UNROM
  .addr msg_unrom
  .byte MAPPER_UNROM_CRAZY
  .addr msg_unrom_crazy
last_mapper_msg = * - mapper_msgs

msg_gnrom:       .byte "G*ROM",0
msg_fme7:        .byte "J*ROM (FME-7)",0
msg_holydiver:   .byte "HOLY DIVER",0
msg_aorom:       .byte "A*ROM",0
msg_bnrom:       .byte "BNROM",0
msg_unrom:       .byte "U*ROM",0
msg_unrom_crazy: .byte "U*ROM (7408)",0
msg_a53:         .byte "INL-ROM (A53)",0
msg_mmc1:        .byte "S*ROM (MMC1)",0
msg_mmc2:        .byte "PNROM (MMC2)",0
msg_mmc3:        .byte "T*ROM (MMC3)",0
msg_mmc3_tlsrom: .byte "T*SROM (MMC3)",0
msg_mmc4:        .byte "F*ROM (MMC4)",0

k_prg_rom:       .byte "K PRG ROM",0
k_prg_ram:       .byte "K "
msg_prg_ram:     .byte "PRG RAM",0
msg_missing:     .byte " MISSING",0
k_chr:           .byte "K CHR R",0
msg_battery:     .byte " + BATTERY",0
msg_ok:          .byte " OK",0
msg_problem:     .byte " PROBLEM",0
msg_detailed:    .byte "DETAILED TEST RESULT: ",0
msg_flashid:     .byte "FLASH ID RESULT:",0

msg_todo:
  .byte "HOLY DIVER BATMAN",LF
  .byte "TOOL FOR TESTING FAMICOM/NES",LF
  .byte "CARTRIDGE BOARD ASSEMBLY",LF
  .byte "COPR. 2013 DAMIAN YERRICK",LF
  .byte "            WWW.PINEIGHT.COM",LF
  .byte "    WWW.INFINITENESLIVES.COM",0

