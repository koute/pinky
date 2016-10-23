CUSTOM_NMI = 1
CUSTOM_IRQ = 1
.include "shell.inc"

zp_res interrupt_count

nmi:    inc interrupt_count
	rti

irq:    inc interrupt_count
	lsr $4015
	rti

main:
	set_test 2,"Interrupts could not be disabled"
	mov SNDCHN,#0           ; disable interrupts
	mov $4010,#0
	mov SNDMODE,#$C0
	mov PPUCTRL,#0
	cli
	mov interrupt_count,#0  ; be sure none occur
	delay_msec_approx 100
	lda interrupt_count
	jne test_failed
	
	set_test 3,"PHP should set bits 4 and 5 on stack"
	lda #0
	pha
	plp
	php
	pla
	and #$30
	cmp #$30
	jne test_failed
	
	set_test 4,"PHP and PLP should preserve bits 7,6,3,2,1,0"
	lda #0
	pha
	plp
	php
	pla
	cmp #$30
	jne test_failed
	
	lda #$FF
	pha
	plp
	php
	pla
	cmp #$FF
	jne test_failed
	
	set_test 5,"PHA should store at $100+S"
	ldx #$F1
	txs
	lda #$12
	pha
	lda #$34
	pha
	lda $1F1
	cmp #$12
	jne test_failed
	lda $1F0
	cmp #$34
	jne test_failed
	
	set_test 6,"JSR should push addr of next instr - 1"
	setb $6FE,$20 ; JSR
	setb $6FF,<:+
	setb $700,>:+
	jmp $6FE
:       pla
	cmp #$00
	jne test_failed
	pla
	cmp #$07
	jne test_failed
	
	set_test 7,"RTS should return to addr+1"
	lda #>:+
	pha
	lda #<:+
	pha
	ldx #0
	rts
	inx
:       inx
	inx
	cpx #1
	jne test_failed
	
	jmp tests_passed
