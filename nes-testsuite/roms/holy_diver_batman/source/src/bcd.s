;
; Binary to decimal conversion for 8-bit and 16-bit numbers
; Copyright 2012 Damian Yerrick
;
; Copying and distribution of this file, with or without
; modification, are permitted in any medium without royalty provided
; the copyright notice and this notice are preserved in all source
; code copies.  This file is offered as-is, without any warranty.
;
.export bcd8bit
.exportzp bcdNum, bcdResult
.export bcdConvert

; This file contains functions to convert binary numbers (base 2)
; to decimal representation (base 10).  Most of the hex to decimal
; routines on 6502.org rely on the MOS Technology 6502's decimal
; mode for ADC and SBC, which was removed from the Ricoh parts.
; These do not use decimal mode.

; The 8-bit routine ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

.macro bcd8bit_iter value
  .local skip
  cmp value
  bcc skip
  sbc value
skip:
  rol highDigits
.endmacro

;;
; Converts a decimal number to two or three BCD digits
; in no more than 80 cycles.
; @param a the number to change
; @return a: low digit; 0: upper digits as nibbles
; No other memory or register is touched.
.proc bcd8bit
highDigits = 0

  ; First clear out two bits of highDigits.  (The conversion will
  ; fill in the other six.)
  asl highDigits
  asl highDigits

  ; Each iteration takes 11 if subtraction occurs or 10 if not.
  ; But if 80 is subtracted, 40 and 20 aren't, and if 200 is
  ; subtracted, 80 is not, and at least one of 40 and 20 is not.
  ; So this part takes up to 6*11-2 cycles.
  bcd8bit_iter #200
  bcd8bit_iter #100
  bcd8bit_iter #80
  bcd8bit_iter #40
  bcd8bit_iter #20
  bcd8bit_iter #10
  rts
.endproc

;;
; Converts a 16-bit number in bcdNum to 5 decimal digits in
; bcdResult.  Unlike some 6502 binary-to-decimal converters, this
; subroutine doesn't use the decimal mode that was removed from
; the 2A03 variant of the 6502 processor.
;
; For each value of n from 4 to 1, it compares the number to 8*10^n,
; then 4*10^n, then 2*10^n, then 1*10^n, each time subtracting if
; possible. After finishing all the comparisons and subtractions in
; each decimal place value, it writes the digit to the output array
; as a byte value in the range [0, 9].  Finally, it writes the
; remainder to element 0.
;
; Extension to 24-bit and larger numbers is straightforward:
; Add a third bcdTable, increase BCD_BITS, and extend the
; trial subtraction.

; Constants _________________________________________________________
; BCD_BITS
;   The highest possible number of bits in the BCD output. Should
;   roughly equal 4 * log10(2) * x, where x is the width in bits
;   of the largest binary number to be put in bcdNum.
; bcdTableLo[y], bcdTableHi[y]
;   Contains (1 << y) converted from BCD to binary.
BCD_BITS = 19

; Variables _________________________________________________________
; bcdNum (input)
;   Number to be converted to decimal (16-bit little endian).
;   Overwritten.
; bcdResult (output)
;   Decimal digits of result (5-digit little endian).
; X
;   Offset of current digit being worked on.
; Y
;   Offset into bcdTable*.
; curDigit
;   The lower nibble holds the digit being constructed.
;   The upper nibble contains a sentinel value; when a 1 is shifted
;   out, the byte is complete and should be copied to result.
;   (This behavior is called a "ring counter".)
;   Overwritten.
; b
;   Low byte of the result of trial subtraction.
;   Overwritten.
bcdNum = 0
bcdResult = 2
curDigit = 7
b = bcdResult

;
; Has been timed to take no more than 670 cycles (6 NTSC scanlines).
;

.proc bcdConvert
  lda #$80 >> ((BCD_BITS - 1) & 3)
  sta curDigit
  ldx #(BCD_BITS - 1) >> 2
  ldy #BCD_BITS - 5

@loop:
  ; Trial subtract this bit to A:b
  sec
  lda bcdNum
  sbc bcdTableLo,y
  sta b
  lda bcdNum+1
  sbc bcdTableHi,y

  ; If A:b > bcdNum then bcdNum = A:b
  bcc @trial_lower
  sta bcdNum+1
  lda b
  sta bcdNum
@trial_lower:

  ; Copy bit from carry into digit and pick up 
  ; end-of-digit sentinel into carry
  rol curDigit
  dey
  bcc @loop

  ; Copy digit into result
  lda curDigit
  sta bcdResult,x
  lda #$10  ; Empty digit; sentinel at 4 bits
  sta curDigit
  ; If there are digits left, do those
  dex
  bne @loop
  lda bcdNum
  sta bcdResult
  rts
.endproc

.segment "RODATA"
bcdTableLo:
  .byt <10, <20, <40, <80
  .byt <100, <200, <400, <800
  .byt <1000, <2000, <4000, <8000
  .byt <10000, <20000, <40000

bcdTableHi:
  .byt >10, >20, >40, >80
  .byt >100, >200, >400, >800
  .byt >1000, >2000, >4000, >8000
  .byt >10000, >20000, >40000

.segment "CODE"

.if 0
.export pctageDigit

;;
; Computes the first digit of num/den, where num < den, and
; multiplies num by 10 for the next digit(s).
.proc pctageDigit
numLo = 0
numHi = 1
denLo = 2
denHi = 3
div4Hi = 4
outDigit = 5

  ; Step 1 of 4: First multiply by 5/4
  lda #0
  sta outDigit
  lda numHi
  lsr a
  sta div4Hi
  lda numLo
  ror a
  ror outDigit
  lsr div4Hi
  ror a
  ror outDigit

  ; At this point, div4Hi:A = numHi:numLo / 4, and the top 2 bits of
  ; outDigit contain the remainder.
  adc numLo
  sta numLo
  lda numHi
  adc div4Hi
  sta numHi
  ; If the addition overflowed, we know it's already greater than
  ; den, and a branch near trialSub will pick up this fact.

  ; At this point, we're all done with div4Hi.  It can be repurposed
  ; to hold a temporary value if needed later.
  ldx #4
  bne trialSub

loop:
  ; The second, third, and fourth iterations involve multiplication
  ; by two, which takes far less code.
  rol outDigit
  rol numLo
  rol numHi

trialSub:
  bcs alreadyGreater
  lda numLo
  cmp denLo
  lda numHi
  sbc denHi
  bcc notGreater
alreadyGreater:
  lda numLo
  sbc denLo
  sta numLo
  lda numHi
  sbc denHi
  sta numHi
  sec
notGreater:
  dex
  bne loop
  lda outDigit
  rol a
  rts
.endproc
.endif
