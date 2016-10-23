; Times a piece of code to clock accuracy, using DMC

.include "sync_dmc.s"

; Theory:
; DMC timer is always running cycles, so if we
; synchronize to a cycle, make the cycle long,
; run some code of unknown duration, we can
; then determine how long it took by where the
; DMC cycle is. We find where it is by starting
; a sample then seeing how long until it finishes.
; We can first find roughly how long until it
; finished, then more finely narrow that down to
; the cycle.

dmc_timer_modulo = 3424
dmc_timer_max    = 2980

; Begins timing section of code
; Preserved: A, X, Y, flags
; Time: ~680-3800 cycles
.align 32
begin_dmc_timer:
	php             ; save
	jsr sync_dmc
	
	; Now switch to lowest rate to give
	; a longer timing range.
	pha
	lda #$00        ; lowest rate
	sta $4010
	pla
	
	nop
	plp             ; restore
	rts

; Returns in XA number of cycles elapsed since call to
; time_code_begin, MOD dmc_timer_modulo. Unreliable if
; result is dmc_timer_max or greater.
.align 64
end_dmc_timer:
	; The arbitrary starting X and Y values for the
	; loops merely set an adjustment added to the
	; final count.
	
	; Restart sample, which will immediately
	; finish since nothing's playing, then
	; start again which will ensure the flag
	; stays set until the second one begins.
	; This means that bit 4 of SNDCHN will be set
	; a fixed amount of time after begin_dmc_timer
	; completed.
	lda #$1F
	sta SNDCHN
	nop
	sta SNDCHN
	
	; Coarse sync
	; Get within a few cycles of when DMC sample finishes.
	; Keep a count since each iter is 16 cycles.
	ldy #-$45
@coarse:
	; 16 cycles/iter
	nop
	lda #$10
	bne :+
:       dey
	bit SNDCHN
	bne @coarse
	
	; DO NOT write to memory. It affects timing.
	
	; Now the DMC cycle timing is at one of
	; 16 relative synchronizations with us.
	
	; Fine sync
	; Each iter takes 3423 cycles, and DMC cycle
	; is 3424 cycles, so bit SNDCHN at end will slowly
	; move backwards relative to DMC cycle until
	; it reads just before sample ends, at which point
	; it will have synchronized exactly.
	ldx #-$2
@sync:
	lda #$1F                ; 2
	sta SNDCHN              ; 4
				; 4 DMC DMA
	
	lda #179                ; delay 3402
:       nop
	nop
	nop
	nop
	nop
	nop
	sec
	sbc #1
	bne :-
	
	inx                     ; 2
	lda #$10                ; 2
	bit SNDCHN              ; 4
	beq @sync               ; 3
	
	;jsr print_y
	;jsr print_x
	
	; Calculate result
	; XA = Y*16 + X
	stx <0
	
	tya
	lsr a
	lsr a
	lsr a
	lsr a
	tax
	
	tya
	asl a
	asl a
	asl a
	asl a
	
	clc
	adc <0
	bcc :+
	inx
:       
	rts
