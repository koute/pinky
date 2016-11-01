;
; Globals for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/morse.h"

.global OAM
.globalzp nmis, irqs, irq_handler, cur_mapper
.global identity
JMP_OPCODE = $4C

; ppuclear.s
.global ppu_clear_nt, ppu_clear_oam, ppu_screen_on, read_pads

; drivers.s
.global load_mapper_driver
.globalzp driver_prg_result, driver_chr_result
.globalzp prg_flash_mfrid, prg_flash_devid, prg_flash_width, prg_flash_size
.globalzp chr_flash_mfrid, chr_flash_devid, chr_flash_width, chr_flash_size

; Bitfields for PRG ROM and RAM test (bit set if fail)
MAPTEST_PRGWIN1   = $01  ; window 1
MAPTEST_PRGWIN2   = $02  ; window 2
MAPTEST_PRGWIN3   = $04  ; window 3
MAPTEST_PRGWIN4   = $08  ; window 4
MAPTEST_WRAMEN    = $10  ; WRAM enable/disable
MAPTEST_WRAMRO    = $20  ; WRAM read-only
MAPTEST_WRAMEN2   = $40  ; second WRAM disable (e.g. SNROM CHR A16)

MAPTEST_CHR       = $01  ; CHR bank tags in normal mode
MAPTEST_CHRALT    = $02  ; other 8K mode (MMC1 4K, MMC3 P=1)
MAPTEST_IRQ       = $10

; mapper_detect.s
.global detect_mapper
.globalzp last_prg_bank

; wrongbanks.s
.global morsebeep, wait_y_frames
.global wrongbank_reset, wrongbank_nmi
.global CONSTANT_00, CONSTANT_FF, CUR_BANK, IS_LAST_BANK
.globalzp TWOCELLS

; bcd.s
.global bcd8bit, bcdConvert

; loadchr.s
.globalzp is_chrrom, last_chr_bank, chr_test_result
.global load_small_font, verify_small_font, detect_chrrom
.global write_chr_bank_tags, read_chr_bank_tags, get_chr_size
.global smallchr_puts_multiline

; wram.s
.global wram_test, round_fillvals
.globalzp has_savedata, has_wram, last_wram_bank, wram_test_result

; boardletter.s
.global get_board_name

; beepcode
.global beepcode_tweet, beepcode_nibble, beepcode_byte
.global beepcode_null, beepcode_ding

.macro crc8_update_0
.local no_eor
  asl a
  bcc no_eor
  eor #$07
no_eor:
.endmacro

LF = 10

MAPPER_NROM = 0
MAPPER_MMC1 = 1
MAPPER_UNROM = 2
MAPPER_CNROM = 3
MAPPER_MMC3 = 4
MAPPER_AOROM = 7
MAPPER_MMC2 = 9
MAPPER_MMC4 = 10
MAPPER_CPROM = 13
MAPPER_A53 = 28
MAPPER_BNROM = 34
MAPPER_GNROM = 66
MAPPER_FME7 = 69
MAPPER_HOLYDIVER = 78
MAPPER_MMC3_TLSROM = 118
MAPPER_UNROM_CRAZY = 180
MAPPER_UNKNOWN = 248

A53_SELECT = $5000
A53_DATA = $8000
A53_MODE = $80
A53_OUTERBANK = $81
A53_CHRBANK = $00
A53_PRGBANK = $01
A53_GAME32K = $00
A53_GAME64K = $10
A53_GAME128K = $20
A53_GAME256K = $30

MMC1_MODE = $8000
MMC1_MIRR0 = $00
MMC1_MIRR1 = $01
MMC1_MIRRV = $02
MMC1_MIRRH = $03
MMC1_PRG32K = $00
MMC1_PRG16K_FIXLO = $08
MMC1_PRG16K_FIXHI = $0c
MMC1_CHR8K = $00
MMC1_CHR4K = $10
MMC1_CHRBANK0 = $A000
MMC1_CHRBANK1 = $C000
MMC1_PRGBANK = $E000

FME7_SELECT = $8000
FME7_DATA = $A000
FME7_CHRBANK  = 0
FME7_PRGBANK  = 8
FME7_PRGBANK_ROM = $00  ; these are OR'd with the bank number
FME7_PRGBANK_OFF = $40  ; but only in FME7_PRGBANK+0
FME7_PRGBANK_RAM = $C0
FME7_MIRR     = 12
FME7_MIRRV = $00
FME7_MIRRH = $01
FME7_MIRR0 = $02
FME7_MIRR1 = $03
FME7_IRQCTRL  = 13
FME7_IRQ_COUNT  = $80
FME7_IRQ_ENABLE = $01
FME7_IRQ_LOW  = 14
FME7_IRQ_HIGH = 15


MMC3_SELECT = $8000
MMC3_DATA = $8001
MMC3_MIRR = $A000  ; FME7 H and V mirroring values work here
MMC3_WRAM_ENABLE = $A001
MMC3_WRAM_OFF = $00
MMC3_WRAM_RW = $80
MMC3_WRAM_RO = $C0
MMC3_IRQ_PERIOD = $C000
MMC3_IRQ_RELOAD = $C001
MMC3_IRQ_DISABLE = $E000
MMC3_IRQ_ENABLE = $E001

MMC2_PRGBANK = $A000
MMC2_CHRBANK0_FD = $B000
MMC2_CHRBANK0_FE = $C000
MMC2_CHRBANK1_FD = $D000
MMC2_CHRBANK1_FE = $E000
MMC2_MIRR = $F000  ; FME7 H and V mirroring values work here too

driver_load_prg_ram_8k = $0400
driver_load_chr_8k = $0403
driver_mapper_test = $0406
