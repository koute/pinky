; Times a piece of code and allows printing the result

.include "dmc_timer.s"

time_code_begin = begin_dmc_timer
time_code_end   = end_dmc_timer

bin2dec_temp = temp
.zeropage
decimal: .res 6
.code

.include "bin2dec_mini16.s"

; Prints X*$100+A as 1-5 digit decimal value
print_dec_xa:
	; Convert to decimal
	sta binary
	stx binary+1
	jsr bin2dec_16bit
	
	; Find first non-zero digit
	ldx #0
:   lda decimal,x
	bne :+
	inx
	cpx #4
	bne :-
	
	; Print remaining digits
:   lda decimal,x
	ora #'0'
	jsr print_char
	inx
	cpx #5
	bne :-
	
	rts
