; Converts 16 bit value to 5 decimal digits.
; See readme for documentation.

; Copyright (C) 2006 Shay Green (I modified source code)
; Copyright (C) 2006 Damian Yerrick
;
; This software is provided 'as-is', without any express or implied
; warranty.  In no event will the authors be held liable for any damages
; arising from the use of this software.
;
; Permission is granted to anyone to use this software for any purpose,
; including commercial applications, and to alter it and redistribute it
; freely, subject to the following restrictions:
;
; 1. The origin of this software must not be misrepresented; you must not
;    claim that you wrote the original software. If you use this software
;    in a product, an acknowledgment in the product documentation would be
;    appreciated but is not required.
; 2. Altered source versions must be plainly marked as such, and must not be
;    misrepresented as being the original software.
; 3. This notice may not be removed or altered from any source distribution.
;
; Damian Yerrick <tepples@spamcop.net>

binary = decimal + 4

; Converts 16-bit value from binary to 5 digits in decimal,
; MOST significant digits first. Decimal must point to
; 6-byte buffer in zero page, and bin2dec_temp to 2-byte
; temporary buffer.
; Time: 570 clocks on average
bin2dec_16bit:
	ldx #-4
	ldy #14
	lda #$20

bin2dec_custom:
	stx bin2dec_temp
	sta bin2dec_temp+1
	clc
@bit:   lda binary
	sbc @lo,y
	tax
	lda binary+1
	sbc @hi,y
	
	bcc :+
	sta binary+1
	stx binary
:   
	rol bin2dec_temp+1
	dey
	bcc @bit
	
	lda bin2dec_temp+1
	ldx bin2dec_temp
	sta <(decimal+4),x
	lda #$10
	inx
	bne bin2dec_custom
	
	rts

@lo:    .byte $09,$13,$27,$4F
	.byte $63,$C7,$8F,$1F
	.byte $E7,$CF,$9F,$3F
	.byte $0F,$1F,$3F

@hi:    .byte $00,$00,$00,$00
	.byte $00,$00,$01,$03
	.byte $03,$07,$0F,$1F
	.byte $27,$4E,$9C
