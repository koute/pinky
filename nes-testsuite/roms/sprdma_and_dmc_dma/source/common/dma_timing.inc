; When memory write occurs during DMC DMA access,
; DMA takes only 3 clocks instead of the usual 4
; (DMC DMA probably gets bad read for that byte).

.include "shell.inc"
.include "code_timer.s"

trim = 127

run_timing:
	jsr console_hide
	setb $4011,$40
	delay_msec 500
	loop_n_times test_,iter
	rts

test_:
	jsr print_a
	pha
	eor #$FF
	pha
	
	setb $4012,<((dmc_sample-$C000)/$40)
	
	jsr pre_test
	
	jsr time_code_begin
	
	; Start DMC
	setb $4015,$10 ; fill sample buffer
	setb $4015,dma*$10
	
	pla
	jsr :+
	pla
	jsr delay_a_25_clocks
	delay 3424+trim+time-overhead-6+18+237+2
	
	jsr time_code_end
	pha
	txa
	pha
	
	jsr post_test
	
	pla
	tax
	pla
	jsr print_dec_xa
	jsr print_newline
	rts
	
:       jsr delay_a_25_clocks
	delay 3085-trim-time-18-237
	jmp test

.align $40
dmc_sample:
	.res $100,$55

.align 32