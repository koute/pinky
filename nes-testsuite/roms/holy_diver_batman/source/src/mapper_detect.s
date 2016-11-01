;
; Mapper detection for Holy Diver Batman
; Copyright 2013 Damian Yerrick
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.include "src/nes.h"
.include "src/ram.h"

MIRRPROBE_0 = %0000
MIRRPROBE_1 = %1111
MIRRPROBE_V = %0101
MIRRPROBE_H = %0011
MIRRPROBE_HINVERT = %1100  ; used by TxSROM detection

.segment "LOWCODE"
;;
; Detects which mapper is present by its nametable mirroring control.
; @return A = one of the following mapper classes
; MAPPER_NROM: fixed mirroring
;   NROM, CNROM, CPROM, GNROM, UNROM-2, UNROM-180
; MAPPER_AOROM: single-screen mirroring with discrete switching
; MAPPER_MMC1: MMC1 (SxROM)
; MAPPER_MMC2: MMC2/MMC4 mirroring (PxROM FxROM)
;   PNROM, FJROM/FKROM
; MAPPER_MMC3: MMC3 mirroring (TxROM, but not TxSROM)
; MAPPER_MMC3_TLSROM: MMC3 (TxSROM, CIRAM A10 = CHR A17)
; MAPPER_FME7: FME-7 mirroring (JxROM, BTR)
; MAPPER_HOLYDIVER: Discrete switching 0=H 1=V (IF-12)
; MAPPER_A53: Action 53 mirroring (INL-ROM v1)
; Further tests after this one refine the guess (see narrow_discrete
; etc.)
; Also want to make sure last 16K is swapped in.
.proc detect_mapper
  ; Lock A53 into BOROM mode (5-bit 32K select)
  lda #A53_MODE
  sta A53_SELECT
  lda #MMC1_PRG32K|MMC1_MIRRV
  sta A53_DATA
  lda #A53_OUTERBANK
  sta A53_SELECT
  ; force last outer bank on A53 or reset MMC1
  lda #$FF  
  sta A53_DATA
  ; and lock this in
  lda #A53_CHRBANK
  sta A53_SELECT

  ; Now test for each mapper by how it handles nametable mirroring
  ; MMC1 first
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRRV|MMC1_CHR8K
  jsr mmc1_set_mode
  jsr write_mirror_probe_vals
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_V
  bne not_mmc1
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRRH|MMC1_CHR8K
  jsr mmc1_set_mode
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_H
  bne not_mmc1
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRR0|MMC1_CHR8K
  jsr mmc1_set_mode
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_0
  bne not_mmc1
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRR1|MMC1_CHR8K
  jsr mmc1_set_mode
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_1
  bne not_mmc1
  jmp finish_init_mmc1
  
not_mmc1:

  ; Detect FME-7 and MMC3 at the same time.  The FME-7 data port is
  ; at the same address as the MMC3 mirroring port, and the same
  ; values (0=V 1=H) work in both.
  lda #FME7_MIRR
  sta FME7_SELECT
  lda #FME7_MIRRV
  sta FME7_DATA
  jsr write_mirror_probe_vals
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_V
  bne not_fme7_mmc3
  lda #FME7_MIRRH
  sta FME7_DATA
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_H
  bne not_fme7_mmc3
  
  ; MMC3 treats the FME-7 value meaning "single screen from CIRAM
  ; $000" as vertical mirroring.  
  lda #FME7_MIRR0
  sta FME7_DATA
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_V
  beq is_mmc3
  cmp #MIRRPROBE_0
  bne not_fme7_mmc3
  lda #$0B  ; Put penult 8K bank into $C000-$DFFF
  sta FME7_SELECT
  lda #$FE
  sta FME7_DATA
  lda #MAPPER_FME7
have_mapper_1:
  rts
is_mmc3:
  lda #6  ; don't crash on INL's subset mapper that fixes P=1
  sta MMC3_SELECT
  lda #$FE
  sta MMC3_DATA
  lda #MAPPER_MMC3
  rts
not_fme7_mmc3:

  ; Look for TLSROM by setting flipped-vertical mirroring, writing
  ; probe values, and reading them back as normal vertical mirroring
  ldx #0
  stx MMC3_SELECT
  lda #$80
  sta MMC3_DATA  ; top half: CIRAM $400
  inx
  stx MMC3_SELECT
  lda #$02
  sta MMC3_DATA  ; bottom half: CIRAM $000
  jsr write_mirror_probe_vals
  lda #$82
  sta MMC3_DATA  ; bottom half: CIRAM $400
  ldx #$00
  stx MMC3_SELECT
  stx MMC3_DATA  ; top half: CIRAM $000
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_HINVERT
  bne not_tlsrom
  lda #6  ; don't crash on INL's subset mapper that fixes P=1
  sta MMC3_SELECT
  lda #$FE
  sta MMC3_DATA
  lda #MAPPER_MMC3_TLSROM
  rts  
not_tlsrom:
  
  ; Look for MMC2/MMC4 (Punch-Out!!, Fire Emblem, Famicom Wars)
  lda #FME7_MIRRV
  sta MMC2_MIRR
  jsr write_mirror_probe_vals
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_V
  bne not_mmc2_mmc4
  lda #FME7_MIRRH
  sta MMC2_MIRR
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_H
  bne not_mmc2_mmc4
  lda #MAPPER_MMC2
  jmp narrow_mmc2
not_mmc2_mmc4:

  ; Detect Action 53 through the same sequence as MMC1, just written
  ; to a different port
  lda #$80
  sta $5000
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRRV
  sta $8000
  jsr write_mirror_probe_vals
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_V
  bne not_a53
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRRH
  sta $8000
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_H
  bne not_a53
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRR0
  sta $8000
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_0
  bne not_a53
  lda #MMC1_PRG16K_FIXHI|MMC1_MIRR1
  sta $8000
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_1
  bne not_a53
  lda #MAPPER_A53
  rts
not_a53:

  ; Finally, detect AOROM (0=single 0 16=single 1), Holy Diver
  ; (8=vert 0=horz), and fixed.  Write to nametables in a sequence
  ; that guarantees that CIRAM $000 = $00 and CIRAM $400 = $01.
  ldx #$20
  ldy #$00
  sty CONSTANT_00
  stx PPUADDR
  sty PPUADDR
  sty PPUDATA
  ldy #$FF
  sty CONSTANT_FF
  jsr write_mirror_probe_vals
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_1
  bne not_aorom
  ldy #$00
  sty CONSTANT_00
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_0
  bne unknown_mapper
  
  ; Mapper 78 has two mirroring variants, each of which has been
  ; given its own NES 2.0 submapper.
  ; 78.3: Holy Diver (mapper-controlled H/V mirroring)
  ; 78.1: Uchuusen Cosmo Carrier (1-screen mirroring)
  ; At this point, we're either on 78.1 or AOROM.  The test ROM isn't
  ; intended to support the Uchuusen Cosmo Carrier board, but even
  ; though NES 2.0 format has been around for 7 years, a lot of
  ; emulators (such as FCEUX, the go-to debugging emulator on Linux)
  ; still as of 2013 assume all #78 games are Uchuusen Cosmo Carrier
  ; unless they have the same SHA-1 as Holy Diver.  So we still need
  ; to distinguish the two and fail if it isn't really AOROM.
  lda #$FF
  sta CONSTANT_FF
  lda #$F0
  sta identity+$F0  ; UCC: page 0; AOROM: page 1
  jsr read_mirror_probe_vals
  cmp #MIRRPROBE_1
  beq is_aorom
  jmp unknown_mapper
is_aorom:
  lda #MAPPER_AOROM
  jmp ff_and_return
not_aorom:

  cmp #MIRRPROBE_H
  bne not_fixed_h
  jmp narrow_discrete
not_fixed_h:

  ; It's either fixed V or Holy Diver.
  ldy #$00
  sty CONSTANT_00
  jsr read_mirror_probe_vals
  ldy #$FF
  sty CONSTANT_FF
  cmp #MIRRPROBE_V
  bne not_fixed_v
  jmp narrow_discrete
not_fixed_v:
  cmp #MIRRPROBE_H
  bne unknown_mapper
  lda #MAPPER_HOLYDIVER
  rts
.endproc
.proc unknown_mapper
  lda #MORSE_M
  ldx #MORSE_I
  ldy #MORSE_R
.endproc
.proc morsebeep_axy
  sta 1
  stx 2
  sty 3
  lda #$04
  sta $4015
  lda #VBLANK_NMI
  sta PPUCTRL
loop:
  lda 1
  jsr morsebeep
  lda 2
  jsr morsebeep
  lda 3
  beq :+
  jsr morsebeep
:
  ; interword space is 4 cells longer than intercharacter
  ldy #2*TWOCELLS
  jsr wait_y_frames
  jmp loop
.endproc



;;
; Sets the MMC1 mode (mirroring, PRG bank size, CHR bank size).
; @param A mode (OR of MMC1_PRG, MMC1_CHR, MMC1_MIRR values)
.proc mmc1_set_mode
  sta MMC1_MODE
  .repeat 4
    lsr a
    sta MMC1_MODE
  .endrepeat
  rts
.endproc

;;
; Writes $00 to $2000 and $01 to $2C00 for mirroring detection.
.proc write_mirror_probe_vals
  ldx #$20
  ldy #$00
  stx PPUADDR
  sty PPUADDR
  sty PPUDATA
  ldx #$2C
  stx PPUADDR
  sty PPUADDR
  iny
  sty PPUDATA
  rts
.endproc

;;
; Read mirroring probe
.proc read_mirror_probe_vals
  ldx #$20  ; src address high
  ldy #$00  ; src address low
  lda #$10  ; ring counter loop: finish once the 1 gets rotated out
readloop:
  pha
  stx PPUADDR
  inx
  sty PPUADDR
  inx
  bit PPUDATA
  inx
  lda PPUDATA
  inx
  lsr a
  pla
  rol a
  bcc readloop
  rts
.endproc

.proc narrow_discrete
  ; Distinguish NROM, BNROM, CNROM, CPROM, GNROM, UNROM-2, UNROM-180
  lda #$00
  sta CONSTANT_00
  lda CUR_BANK
  cmp #3
  bne not_unrom180
  lda #MAPPER_UNROM_CRAZY
  jmp ff_and_return
not_unrom180:

  cmp #7
  beq narrow_32k
  lda #MAPPER_UNROM
  rts
narrow_32k:

  ; Distinguish NROM, BNROM, CNROM, CPROM, GNROM
  lda #$FF
  sta CONSTANT_FF
  lda IS_LAST_BANK
  bne :+
  jmp unknown_mapper
:
  ; GNROM: [PPPP CCCC]
  ; If F0 doesn't go to the last bank, it's BNROM.
  lda #$F0
  sta CONSTANT_F0
  lda IS_LAST_BANK
  bne not_bnrom
  lda #MAPPER_BNROM
  jmp ff_and_return
not_bnrom:
  ; At this point it's either CPROM, GNROM, or one of GNROM's
  ; undersize subsets (NROM and CNROM).
  ; TO DO: rule out CPROM
  lda #MAPPER_GNROM
  rts
.endproc

.proc ff_and_return
  pha
  lda #$FF
  sta CONSTANT_FF
  pla
  rts
.endproc

;;
; Distinguishes MMC2 ($A000 swaps 8K at $8000-$9FFF)
; from MMC4 ($A000 swaps 16K at $8000-$BFFF)
.proc narrow_mmc2
  lda #1
  sta MMC2_PRGBANK
  ; MMC2: 02 03 0A 0B 0C 0D 0E 0F
  ; MMC4: 04 05 06 07 0C 0D 0E 0F
  lda CUR_BANK-$6000
  cmp #4
  ; carry clear for MMC2, set for MMC4
  lda #MAPPER_MMC2
  bcc is_mmc2
  lda #MAPPER_MMC4
is_mmc2:
  rts
.endproc

; MMC1 is kind of complicated.  On SUROM and SXROM, just writing
; $FF to reset maps the last 16K of the current 256K outer bank
; into $C000-$FFFF, but the outer bank may not be the last bank.
; So first we have to get to the last bank to see if we need to work
; around SUROM protection.
.proc finish_init_mmc1
  ; Switch to PRG ROM bank $0E, disabling MMC1B/C protection
  lda #$0F
  ldy #5
:
  sta $E000
  lsr a
  dey
  bne :-

  ; Switch to the last CHR ROM bank (and outer bank)
  lda #MAPPER_MMC1
  ldy #5
:
  sta $A000
  dey
  bne :-

  ldy IS_LAST_BANK
  beq surom_fail

  ; At this point the last bank is ready, and the MMC1 driver uses
  ; the last bank to decide whether to use SNROM protection.
  rts
surom_fail:
  lda #MORSE_S
  ldx #MORSE_U
  ; Y is already 0
  jmp morsebeep_axy
.endproc


.segment "RODATA"
CONSTANT_F0: .byte $F0

