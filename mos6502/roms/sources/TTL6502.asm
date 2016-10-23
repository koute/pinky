; TTL-6502 program


.eq Test	= $03
.eq TmpZP0	= $10
.eq TmpAB0	= $1000


.ba $E000

Reset
		nop


;**  Check if JMP works fine
		jmp	L999

;* Should not be executed at all
		nop
		sec
L000c
		bcs	L000c
		
		nop
L000
		jmp	M372
		
		nop
L000a
		clc
		lda	#0
		rts
		
		nop

		
;**  Test branches with all flags set
L005
		lda	#1
		sta	Test

		ldx	#$FF
		txs

		lda	#$FF
		pha
		plp			; set all flags

;!! Remark: if an error occurs, don't forget that one of the above instructions
;           could have been the cause instead.
L001
		bcc	L001
L002
		bne	L002
L003
		bvc	L003
L004
		bpl	L004

		bmi	L006		; must branch, else error
L004a
		jmp	L004a		; error, ->
L006
		beq	L007		; must branch, else error
L006a
		jmp	L006a		; error, ->
L007
		bvs	L008		; must branch, else error
L007a
		jmp	L007a		; error, ->
L008
		bcs	L010		; must branch, else error

; Branch shouldn't have happened,
L008a
		jmp	L008a

;**  Test branches with all flags reset
L010
		lda	#2
		sta	Test

		lda	#0
		pha
		plp			; reset all flags
L011
		bcs	L011
L012
		beq	L012
L013
		bvs	L013
L014
		bmi	L014

		bcc	L015		; must branch, else error
L014a
		jmp	L014a		; error, ->
L015
		bne	L016		; must branch, else error
L015a
		jmp	L015a		; error, ->
L016
		bvc	L017		; must branch, else error
L016a
		jmp	L016a		; error, ->
L017
		bpl	L020		; must branch, else error
L017a
		jmp	L017a		; error, ->


;**  Test SEC, CLC and CLV
L020

		lda	#3
		sta	Test
L020c
		beq	L020c		; must skip, else error

		bne	L020e
L020d
		jmp	L020d
L020e
		sec
L020a
		bcc	L020a		; error, ->

		bcs	L021		; must branch, else error
L020b
		jmp	L020b		; error, ->

L021
		clc
L021a
		bcs	L021a		; error, ->

		bcc	L022		; must branch, else error
L021b
		jmp	L021b		; error, ->

L022
		clv
L022a
		bvs	L022a		; error, ->

		bvc	L030		; must branch, else error
L022b
		jmp	L022b		; error, ->


;**  Test LDA/X/Y #n, STA/X/Y $xx/$xxxx
L030
		lda	#4
		sta	Test

		lda	#0
L031
		bne	L031		; error, ->
L032
		bmi	L032		; error, ->

		sta	TmpZP0
		sta	TmpAB0

		lda	#$80
L033
		beq	L033		; error, ->
L034
		bpl	L034		; error, ->

		sta	TmpZP0+1
		sta	TmpAB0+1

		ldx	#0
L035
		bne	L035		; error, ->
L036
		bmi	L036		; error, ->

		stx	TmpZP0+2
		stx	TmpAB0+2

		ldx	#$80
L037
		beq	L037		; error, ->
L038
		bpl	L038		; error, ->

		stx	TmpZP0+3
		stx	TmpAB0+3

		ldy	#0
L039a
		bne	L039a		; error, ->
L039b
		bmi	L039b		; error, ->

		sty	TmpZP0+4
		sty	TmpAB0+4

		ldy	#$80
L039c
		beq	L039c		; error, ->
L039d
		bpl	L039d		; error, ->

		sty	TmpZP0+5
		sty	TmpAB0+5


;**  Test LDA/X/Y $xx
		lda	#5
		sta	Test

		lda	TmpZP0
L039e
		bne	L039e		; error, ->
L039f
		bmi	L039f		; error, ->


		lda	TmpZP0+1
L039g
		beq	L039g		; error, ->
L039h
		bpl	L039h		; error, ->


		ldx	TmpZP0
L039i
		bne	L039i		; error, ->
L039j
		bmi	L039j		; error, ->


		ldx	TmpZP0+1
L039k
		beq	L039k		; error, ->
L039l
		bpl	L039l		; error, ->


		ldy	TmpZP0
L039m
		bne	L039m		; error, ->
L039n
		bmi	L039n		; error, ->


		ldy	TmpZP0+1
L039o
		beq	L039o		; error, ->
L039p
		bpl	L039p		; error, ->


;**  Test branch around page border
		clc
		bcc	L40m
L40a
		jmp	L40a

L40b
		sec
		bcs	L40z
		
.fb $EA, 14

L40m		bcc	L40b
						

;**  Test LDA/X/Y $xxxx
L40z
		lda	#6
		sta	Test

		lda	TmpAB0
L041
		bne	L041		; error, ->
L042
		bmi	L042		; error, ->


		lda	TmpAB0+1
L043
		beq	L043		; error, ->
L044
		bpl	L044		; error, ->


		ldx	TmpAB0
L044a
		bne	L044a		; error, ->
L044b
		bmi	L044b		; error, ->


		ldx	TmpAB0+1
L044c
		beq	L044c		; error, ->
L044d
		bpl	L044d		; error, ->

		ldy	TmpAB0
L044e
		bne	L044e		; error, ->
L044f
		bmi	L044f		; error, ->

		ldy	TmpAB0+1
L044g
		beq	L044g		; error, ->
L044h
		bpl	L044h		; error, ->


;**  Test CMP/CPX/CPY
;* register = value
		ldx	#0
		ldy	#0
		lda	#7
		sta	Test
		sta	TmpZP0
		sta	TmpAB0

		cmp	#7
L051
		bne	L051		; error, ->
L052
		bcc	L052		; error, ->
L053
		bmi	L053		; error, ->


		cmp	TmpZP0
L051a
		bne	L051a		; error, ->
L052a
		bcc	L052a		; error, ->
L053a
		bmi	L053a		; error, ->


		cmp	TmpAB0
L051b
		bne	L051b		; error, ->
L052b
		bcc	L052b		; error, ->
L053b
		bmi	L053b		; error, ->


		ldx	#7
		cpx	#7
L051c
		bne	L051c		; error, ->
L052c
		bcc	L052c		; error, ->
L053c
		bmi	L053c		; error, ->


		cpx	TmpZP0
L051d
		bne	L051d		; error, ->
L052d
		bcc	L052d		; error, ->
L053d
		bmi	L053d		; error, ->


		cpx	TmpAB0
L051e
		bne	L051e		; error, ->
L052e
		bcc	L052e		; error, ->
L053e
		bmi	L053e		; error, ->


		ldy	#7
		cpy	#7
L051f
		bne	L051f		; error, ->
L052f
		bcc	L052f		; error, ->
L053f
		bmi	L053f		; error, ->


		cpy	TmpZP0
L051g
		bne	L051g		; error, ->
L052g
		bcc	L052g		; error, ->
L053g
		bmi	L053g		; error, ->


		cpy	TmpAB0
L051h
		bne	L051h		; error, ->
L052h
		bcc	L052h		; error, ->
L053h
		bmi	L053h		; error, ->

;* register > memory
		lda	#8
		sta	Test


		cmp	#7
L061
		beq	L061		; error, ->
L062
		bcc	L062		; error, ->
L063
		bmi	L063		; error, ->


		cmp	TmpZP0
L061a
		beq	L061a		; error, ->
L062a
		bcc	L062a		; error, ->
L063a
		bmi	L063a		; error, ->


		cmp	TmpAB0
L061b
		beq	L061b		; error, ->
L062b
		bcc	L062b		; error, ->
L063b
		bmi	L063b		; error, ->


		ldx	#8
		cpx	#7
L061c
		beq	L061c		; error, ->
L062c
		bcc	L062c		; error, ->
L063c
		bmi	L063c		; error, ->


		cpx	TmpZP0
L061d
		beq	L061d		; error, ->
L062d
		bcc	L062d		; error, ->
L063d
		bmi	L063d		; error, ->


		cpx	TmpAB0
L061e
		beq	L061e		; error, ->
L062e
		bcc	L062e		; error, ->
L063e
		bmi	L063e		; error, ->


		ldy	#8
		cpy	#7
L061f
		beq	L061f		; error, ->
L062f
		bcc	L062f		; error, ->
L063f
		bmi	L063f		; error, ->


		cpy	TmpZP0
L061g
		beq	L061g		; error, ->
L062g
		bcc	L062g		; error, ->
L063g
		bmi	L063g		; error, ->


		cpy	TmpAB0
L061h
		beq	L061h		; error, ->
L062h
		bcc	L062h		; error, ->
L063h
		bmi	L063h		; error, ->


;* register < memory
		lda	#8
		sta	Test

		lda	#5
		cmp	#7
L071
		beq	L071		; error, ->
L072
		bcs	L072		; error, ->
L073
		bpl	L073		; error, ->


		cmp	TmpZP0
L071a
		beq	L071a		; error, ->
L072a
		bcs	L072a		; error, ->
L073a
		bpl	L073a		; error, ->


		ldx	#5
		cpx	#7
L071b
		beq	L071b		; error, ->
L072b
		bcs	L072b		; error, ->
L073b
		bpl	L073b		; error, ->


		cpx	TmpZP0
L071c
		beq	L071c		; error, ->
L072c
		bcs	L072c		; error, ->
L073c
		bpl	L073c		; error, ->


		cpx	TmpAB0
L071d
		beq	L071d		; error, ->
L072d
		bcs	L072d		; error, ->
L073d
		bpl	L073d		; error, ->


		ldy	#5
		cpy	#7
L071e
		beq	L071e		; error, ->
L072e
		bcs	L072e		; error, ->
L073e
		bpl	L073e		; error, ->


		cpy	TmpZP0
L071f
		beq	L071f		; error, ->
L072f
		bcs	L072f		; error, ->
L073f
		bpl	L073f		; error, ->


		cpy	TmpAB0
L071g
		beq	L071g		; error, ->
L072g
		bcs	L072g		; error, ->
L073g
		bpl	L073g		; error, ->


;**  TAX, TXA
		lda	#9
		sta	Test

		ldx	#0
		lda	#$AA
		tax
L081
		bpl	L081		; error, ->
L082
		beq	L082		; error, ->

		cpx	#$AA
L083
		bne	L083		; error, ->


		lda	#$0A
		sta	Test

		txa
L084
		bpl	L084		; error, ->
L085
		beq	L085		; error, ->

		cmp	#$AA
L086
		bne	L086		; error, ->


		lda	#$0B
		sta	Test

		lda	#$55
		tax
L087
		bmi	L087		; error, ->
L088
		beq	L088		; error, ->

		cpx	#$55
L089
		bne	L089		; error, ->

		lda	#$0C
		sta	Test

		txa
L089a
		bmi	L089a		; error, ->
L089b
		beq	L089b		; error, ->


		cmp	#$55
L089c
		bne	L089c		; error, ->


		lda	#$0D
		sta	Test

		lda	#0
		tax
L089d
		bmi	L089d		; error, ->
L089e
		bne	L089e		; error, ->


		cpx	#0
L089f
		bne	L089f		; error, ->


		lda	#$0E
		sta	Test

		txa
L089g
		bmi	L089g		; error, ->
L089h
		bne	L089h		; error, ->


		cmp	#0
L089i
		bne	L089i		; error, ->


;**  TAY, TYA
		lda	#$0F
		sta	Test

		ldy	#0
		lda	#$AA
		tay
L091
		bpl	L091		; error, ->
L092
		beq	L092		; error, ->


		cpy	#$AA
L093
		bne	L093		; error, ->


		lda	#$10
		sta	Test

		tya
L091a
		bpl	L091a		; error, ->
L092a
		beq	L092a		; error, ->


		cmp	#$AA
L093a
		bne	L093a		; error, ->


		lda	#$11
		sta	Test

		lda	#$55
		tay
L091b
		bmi	L091b		; error, ->
L092b
		beq	L092b		; error, ->


		cpy	#$55
L093b
		bne	L093b		; error, ->


		lda	#$12
		sta	Test

		tya
L091c
		bmi	L091c		; error, ->
L092c
		beq	L092c		; error, ->


		cmp	#$55
L093c
		bne	L093c		; error, ->


		lda	#$13
		sta	Test

		lda	#0
		tay
L091d
		bmi	L091d		; error, ->
L092d
		bne	L092d		; error, ->


		cpy	#0
L093d
		bne	L093d		; error, ->


		lda	#$14
		sta	Test

		tya
L091e
		bmi	L091		; error, ->
L092e
		bne	L092e		; error, ->


		cmp	#0
L093e
		bne	L093e		; error, ->


;**  TXS, TSX
		lda	#$15
		sta	Test

		ldx	#$AA
		txs
L101
		bpl	L101		; error, ->
L102
		beq	L102		; error, ->


		ldx	#$00
		tsx
L103
		bpl	L103		; error, ->
L104
		beq	L104		; error, ->


		cpx	#$AA
L105
		bne	L105		; error, ->


		lda	#$16
		sta	Test

		ldx	#$55
		txs
L103a
		bmi	L103a		; error, ->
L104a
		beq	L104a		; error, ->


		ldx	#$00
		tsx
L103b
		bmi	L103b		; error, ->
L104b
		beq	L104b		; error, ->


		cpx	#$55
L105b
		bne	L105b		; error, ->


		lda	#$17
		sta	Test

		ldx	#0
		txs
L103c
		bmi	L103c		; error, ->
L104c
		bne	L104c		; error, ->


		ldx	#$55
		tsx
L103d
		bmi	L103d		; error, ->
L104d
		bne	L104d		; error, ->


		cpx	#0
L105d
		bne	L105d		; error, ->


;**  PHA, PLA
		lda	#$17
		sta	Test

		lda	#$AA
		pha
		pla
L111
		beq	L111		; error, ->
L112
		bpl	L112		; error, ->


		lda	#$18
		sta	Test

		lda	#$55
		pha
		pla
L113
		beq	L113		; error, ->
L114
		bmi	L114		; error, ->


		lda	#$19
		sta	Test

		lda	#0
		pha
		pla
L115
		bne	L115		; error, ->
L116
		bmi	L116		; error, ->


;**  DEX
		lda	#$1A
		sta	Test

		ldx	#$81
		dex
L121
		beq	L121		; error, ->
L122
		bpl	L122		; error, ->


		dex
L121a
		beq	L121a		; error, ->
L122a
		bmi	L122a		; error, ->


		lda	#$1B
		sta	Test

		ldx	#2
		dex
L121b
		beq	L121b		; error, ->
L122b
		bmi	L122b		; error, ->


		dex
L121c
		bne	L121c		; error, ->
L122c
		bmi	L122c		; error, ->


		dex
L121d
		beq	L121d		; error, ->
L122d
		bpl	L122d		; error, ->


;**  INX
		lda	#$1C
		sta	Test

		ldx	#$FE
		inx
L121e
		beq	L121e		; error, ->
L122e
		bpl	L122e		; error, ->


		inx
L121f
		bne	L121f		; error, ->
L122f
		bmi	L122f		; error, ->


		inx
L121g
		beq	L121g		; error, ->
L122g
		bmi	L122g		; error, ->


		lda	#$1D
		sta	Test

		ldx	#$7E
		inx
L121h
		beq	L121h		; error, ->
L122h
		bmi	L122h		; error, ->


		inx
L121i
		beq	L121i		; error, ->
L122i
		bpl	L122i		; error, ->

;**  DEY
		lda	#$1E
		sta	Test

		ldy	#$81
		dey
L131
		beq	L131		; error, ->
L132
		bpl	L132		; error, ->


		dey
L131a
		beq	L131a		; error, ->
L132a
		bmi	L132a		; error, ->


		lda	#$1F
		sta	Test

		ldy	#2
		dey
L131b
		beq	L131b		; error, ->
L132b
		bmi	L132b		; error, ->


		dey
L131c
		bne	L131c		; error, ->
L132c
		bmi	L132c		; error, ->


		dey
L131d
		beq	L131d		; error, ->
L132d
		bpl	L132d		; error, ->

;**  INY
		lda	#$20
		sta	Test

		ldy	#$FE
		iny
L131e
		beq	L131e		; error, ->
L132e
		bpl	L132e		; error, ->


		iny
L131f
		bne	L131f		; error, ->
L132f
		bmi	L132f		; error, ->


		iny
L131g
		beq	L131g		; error, ->
L132g
		bmi	L132g		; error, ->


		lda	#$21
		sta	Test

		ldy	#$7E
		iny
L131h
		beq	L131h		; error, ->
L132h
		bmi	L132h		; error, ->


		iny
L131i
		beq	L131i		; error, ->
L132i
		bpl	L132i		; error, ->


;**  LDA indexed  (1)
;* LDA $xx,X
		lda	#$22
		sta	Test

		lda	#$A3
		sta	$50

		ldx	#$30
		lda	$20,X
L141
		beq	L141		; error, ->
L142
		bpl	L142		; error, ->


		cmp	#$A3
L143
		bne	L143		; error, ->


		lda	#$23
		sta	Test

		lda	#$A9
		sta	$20

		lda	$F0,X
L141a
		beq	L141a		; error, ->
L142a
		bpl	L142a		; error, ->


		cmp	#$A9
L143a
		bne	L143a		; error, ->


		lda	#$24
		sta	Test

		lda	#$33
		sta	$50

		lda	$20,X
L141b
		beq	L141b		; error, ->
L142b
		bmi	L142b		; error, ->


		cmp	#$33
L143b
		bne	L143b		; error, ->


		lda	#$25
		sta	Test

		lda	#$35
		sta	$20

		lda	$F0,X
L141c
		beq	L141c		; error, ->
L142c
		bmi	L142c		; error, ->


		cmp	#$35
L143c
		bne	L143c		; error, ->


		lda	#$26
		sta	Test

		lda	#0
		sta	$50

		lda	$20,X
L141d
		bne	L141d		; error, ->
L142d
		bmi	L142d		; error, ->


		cmp	#0
L143d
		bne	L143d		; error, ->


		lda	#$27
		sta	Test

		lda	#0
		sta	$20

		lda	$F0,X
L141e
		bne	L141e		; error, ->
L142e
		bmi	L142e		; error, ->


		cmp	#0
L143e
		bne	L143e		; error, ->


;* LDA $xxxx,X
		lda	#$28
		sta	Test

		lda	#$AA
		sta	TmpAB0+$50

		ldx	#$30
		lda	TmpAB0+$20,X
L151
		beq	L151		; error, ->
L152
		bpl	L152		; error, ->


		cmp	#$AA
L153
		bne	L153		; error, ->


		lda	#$29
		sta	Test

		lda	#$33
		sta	TmpAB0+$50

		lda	TmpAB0+$20,X
L151a
		beq	L151a		; error, ->
L152a
		bmi	L152a		; error, ->


		cmp	#$33
L153a
		bne	L153a		; error, ->


		lda	#$2A
		sta	Test

		lda	#0
		sta	TmpAB0+$50

		lda	TmpAB0+$20,X
L151b
		bne	L151b		; error, ->
L152b
		bmi	L152b		; error, ->


		cmp	#0
L153b
		bne	L153b		; error, ->


;* LDA $xxxx,Y
		lda	#$2B
		sta	Test

		lda	#$AA
		sta	TmpAB0+$50

		ldy	#$30
		lda	TmpAB0+$20,Y
L161
		beq	L161		; error, ->
L162
		bpl	L162		; error, ->


		cmp	#$AA
L163
		bne	L163		; error, ->


		lda	#$2C
		sta	Test

		lda	#$33
		sta	TmpAB0+$50

		lda	TmpAB0+$20,Y
L161a
		beq	L161a		; error, ->
L162a
		bmi	L162a		; error, ->


		cmp	#$33
L163a
 		bne	L163a		; error, ->


		lda	#$2D
		sta	Test

		lda	#0
		sta	TmpAB0+$50

		lda	TmpAB0+$20,Y
L161b
		bne	L161b		; error, ->
L162b
		bmi	L162b		; error, ->


		cmp	#0
L163b
		bne	L163b		; error, ->

			
;**  STA indexed  (1)
;* STA $xx,X
		lda	#$2E
		sta	Test
		sta	$50
		sta	$20

		ldx	#$30
		lda	#$CA
		sta	$20,X

		lda	$50
		cmp	#$CA
L171
		bne	L171		; error, ->


		lda	#$2F
		sta	Test

		lda	#$AC
		sta	$F0,X

		lda	$20
		cmp	#$AC
L172
		bne	L172		; error, ->


		lda	#$30
		sta	Test

		lda	#$5A
		sta	$20,X

		lda	$50
		cmp	#$5A
L173
		bne	L173		; error, ->


		lda	#$31
		sta	Test

		lda	#$5C
		sta	$F0,X

		lda	$20
		cmp	#$5C
L174
		bne	L174		; error, ->


		lda	#$32
		sta	Test

		lda	#0
		sta	$20,X

		lda	$50
L175
		bne	L175		; error, ->


		lda	#$33
		sta	Test

		lda	#0
		sta	$F0,X

		lda	$20
L176
		bne	L176		; error, ->


;* STA $xxxx,X
		lda	#$34
		sta	Test
		sta	TmpAB0+$50

		lda	#$AA
		sta	TmpAB0+$20,X

		lda	TmpAB0+$50
		cmp	#$AA
L177
		bne	L177		; error, ->


		lda	#$35
		sta	Test
		sta	TmpAB0+$50

		lda	#$55
		sta	TmpAB0+$20,X

		lda	TmpAB0+$50
		cmp	#$55
L178
		bne	L178		; error, ->


		lda	#$36
		sta	Test
		sta	TmpAB0+$50

		lda	#0
		sta	TmpAB0+$20,X

		lda	TmpAB0+$50
L179
		bne	L179		; error, ->


;* STA $xxxx,Y
		lda	#$37
		sta	Test
		sta	TmpAB0+$50

		lda	#$AA
		sta	TmpAB0+$20,Y

		lda	TmpAB0+$50
		cmp	#$AA
L179a
		bne	L179a		; error, ->


		lda	#$38
		sta	Test
		sta	TmpAB0+$50

		lda	#$55
		sta	TmpAB0+$20,X

		lda	TmpAB0+$50
		cmp	#$55
L179b
		bne	L179b		; error, ->


		lda	#$39
		sta	Test
		sta	TmpAB0+$50

		lda	#0
		sta	TmpAB0+$20,X

		lda	TmpAB0+$50
L179c
		bne	L179c		; error, ->


;**  LDA indexed  (2)
;* LDA ($xx,X)
		lda	#$3A
		sta	Test

		lda	#$AA
		sta	$50
		sta	TmpAB0+$AA

		lda	#$10
		sta	$51

		ldx	#$30
		lda	($20,X)
L181
		beq	L181		; error, ->
L182
		bpl	L182		; error, ->


		cmp	#$AA
L183
		bne	L183		; error, ->


		lda	#$3B
		sta	Test

		lda	#$AA
		sta	$20

		lda	#$10
		sta	$21

		lda	($F0,X)
L181a
		beq	L181a		; error, ->
L182a
		bpl	L182a		; error, ->


		cmp	#$AA
L183a
		bne	L183a		; error, ->


		lda	#$3C
		sta	Test

		lda	#$33
		sta	TmpAB0+$AA

		lda	($20,X)
L181b
		beq	L181b		; error, ->
L182b
		bmi	L182b		; error, ->


		cmp	#$33
L183b
		bne	L183b		; error, ->


		lda	#$3D
		sta	Test

		lda	#$33
		sta	TmpAB0+$AA

		lda	($F0,X)
L181c
		beq	L181c		; error, ->
L182c
		bmi	L182c		; error, ->


		cmp	#$33
L183c
		bne	L183c		; error, ->


		lda	#$3E
		sta	Test

		lda	#0
		sta	TmpAB0+$AA

		lda	($20,X)
L181d
		bne	L181d		; error, ->
L182d
		bmi	L182d		; error, ->


		cmp	#0
L183d
		bne	L183d		; error, ->


		lda	#$3F
		sta	Test

		lda	#0
		sta	TmpAB0+$AA

		lda	($F0,X)
L181e
		bne	L181e		; error, ->
L182e
		bmi	L182e		; error, ->


		cmp	#0
L183e
		bne	L183e		; error, ->


;* LDA ($xx),Y
		lda	#$40
		sta	Test

		lda	#$AA
		sta	TmpAB0+$AA

		lda	#0
		sta	$50

		lda	#$10
		sta	$51

		ldy	#$AA
		lda	($50),Y
L191
		beq	L191		; error, ->
L192
		bpl	L192		; error, ->


		cmp	#$AA
L193
		bne	L193		; error, ->


		lda	#$41
		sta	Test

		lda	#$55
		sta	TmpAB0+$AA

		lda	($50),Y
L194
		beq	L194		; error, ->
L195
		bmi	L195		; error, ->


		cmp	#$55
L196
		bne	L196		; error, ->


		lda	#$42
		sta	Test

		lda	#0
		sta	TmpAB0+$AA

		lda	($50),Y
L197
		bne	L197		; error, ->
L198
		bmi	L198		; error, ->


		cmp	#0
L199
		bne	L199		; error, ->


;**  STA indexed  (2)
;* STA ($xx,X)
		lda	#$43
		sta	Test
		sta	TmpAB0+$AA

		lda	#$10
		sta	$51

		lda	#$AA
		sta	$50

		ldx	#$30
		sta	($20,X)

		lda	TmpAB0+$AA
		cmp	#$AA
L201
		bne	L201		; error, ->


		lda	#$44
		sta	Test
		sta	TmpAB0+$AA

		lda	#$AA
		sta	$20

		lda	#$10
		sta	$21

		lda	#$AA
		sta	($F0,X)

		lda	TmpAB0+$AA
		cmp	#$AA
L202
		bne	L202		; error, ->


		lda	#$45
		sta	Test
		sta	TmpAB0+$AA

		lda	#$55
		sta	($20,X)

		lda	TmpAB0+$AA
		cmp	#$55
L203
		bne	L203		; error, ->


		lda	#$46
		sta	Test
		sta	TmpAB0+$AA

		lda	#$55
		sta	($F0,X)

		lda	TmpAB0+$AA
		cmp	#$55
L204
		bne	L204		; error, ->


		lda	#$47
		sta	Test
		sta	TmpAB0+$AA

		lda	#0
		sta	($20,X)

		lda	TmpAB0+$AA
L205
		bne	L205		; error, ->


		lda	#$47
		sta	Test
		sta	TmpAB0+$AA

		lda	#0
		sta	($F0,X)

		lda	TmpAB0+$AA
L206
		bne	L206		; error, ->


;* STA ($xx),Y
		lda	#$49
		sta	Test
		sta	TmpAB0+$AA

		lda	#$10
		sta	$19

		lda	#0
		sta	$18

		ldy	#$AA
		lda	#$AA
		sta	($18),Y

		lda	TmpAB0+$AA
		cmp	#$AA
L211
		bne	L211		; error, ->


		lda	#$4A
		sta	Test
		sta	TmpAB0+$AA
 
		lda	#$55
		sta	($18),Y

		lda	TmpAB0+$AA
		cmp	#$55
L212
		bne	L212		; error, ->

		lda	#$56 ;4B
		sta	Test
		sta	TmpAB0+$AA

		lda	#0
		sta	($18),Y

		lda	TmpAB0+$AA
L213
		bne	L213		; error, ->


;**  ADC  (1)
;  CLC 		; $30 + $30 = $60, returns C = 0, returns V = 0
;  LDA #$30
;  ADC #$30

		lda	#$4C
		sta	Test
		
		cld

		lda	#$30
		sta	TmpZP0
		sta	TmpAB0

		clc
		adc	#$30
L221
		bcs	L221		; error, ->
L222
		bvs	L222		; error, ->
L223
		beq	L223		; error, ->
L224
		bmi	L224		; error, ->

		cmp	#$60
L225
		bne	L225		; error, ->


		lda	#$4D
		sta	Test

		clc
		lda	#$30
		adc	TmpZP0
L221a
		bcs	L221a		; error, ->
L222a
		bvs	L222a		; error, ->
L223a
		beq	L223a		; error, ->
L224a
		bmi	L224a		; error, ->


		cmp	#$60
L225a
		bne	L225a		; error, ->


		lda	#$4E
		sta	Test

		clc
		lda	#$30
		adc	TmpAB0
L221b
		bcs	L221b		; error, ->
L222b
		bvs	L222b		; error, ->
L223b
		beq	L223b		; error, ->
L224b
		bmi	L224b		; error, ->


		cmp	#$60
L225b
		bne	L225b		; error, ->


		lda	#$4F
		sta	Test

		lda	#$30
		tax
		sta	$30,X

		clc
		adc	$30,X
L221c
		bcs	L221c		; error, ->
L222c
		bvs	L222c		; error, ->
L223c
		beq	L223c		; error, ->
L224c
		bmi	L224c		; error, ->


		cmp	#$60
L225c
		bne	L225c		; error, ->


		lda	#$50
		sta	Test

		lda	#$30
		sta	TmpAB0,X

		clc
		adc	TmpAB0,X
L221d
		bcs	L221d		; error, ->
L222d
		bvs	L222d		; error, ->
L223d
		beq	L223d		; error, ->
L224d
		bmi	L224d		; error, ->


		cmp	#$60
L225d
		bne	L225d		; error, ->


		lda	#$51
		sta	Test

		lda	#$30
		tay
		sta	TmpAB0,Y

		clc
		adc	TmpAB0,Y
L221e
		bcs	L221e		; error, ->
L222e
		bvs	L222e		; error, ->
L223e
		beq	L223e		; error, ->
L224e
		bmi	L224e		; error, ->


		cmp	#$60
L225e
		bne	L225e		; error, ->


		lda	#$52
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		ldx	#$30
		txa
		sta	($20,X)

		clc
		adc	($20,X)
L221f
		bcs	L221f		; error, ->
L222f
		bvs	L222f		; error, ->
L223f
		beq	L223f		; error, ->
L224f
		bmi	L224f		; error, ->


		cmp	#$60
L225f
		bne	L225f		; error, ->


		lda	#$53
		sta	Test

		lda	#$30
		sta	($F0,X)

		clc
		adc	($F0,X)
L221g
		bcs	L221g		; error, ->
L222g
		bvs	L222g		; error, ->
L223g
		beq	L223g		; error, ->
L224g
		bmi	L224g		; error, ->


		cmp	#$60
L225g
		bne	L225g		; error, ->


;  CLC 		; $30 + $D0 = $00, returns C = 1, returns V = 0
;  LDA #$30
;  ADC #$D0

		lda	#$54
		sta	Test

		lda	#$D0
		sta	TmpZP0
		sta	TmpAB0

		clc
		lda	#$30
		adc	#$D0
L231
		bcc	L231		; error, ->
L232
		bvs	L232		; error, ->
L233
		bne	L233		; error, ->
L234
		bmi	L234		; error, ->


		cmp	#0
L235
		bne	L235		; error, ->


		lda	#$55
		sta	Test

		clc
		lda	#$30
		adc	TmpZP0
L231a
		bcc	L231a		; error, ->
L232a
		bvs	L232a		; error, ->
L233a
		bne	L233a		; error, ->
L234a
		bmi	L234a		; error, ->


		cmp	#0
L235a
		bne	L235a		; error, ->


		lda	#$56
		sta	Test

		clc
		lda	#$30
		adc	TmpAB0
L231b
		bcc	L231b		; error, ->
L232b
		bvs	L232b		; error, ->
L233b
		bne	L233b		; error, ->
L234b
		bmi	L234b		; error, ->


		cmp	#0
L235b
		bne	L235b		; error, ->


		lda	#$57
		sta	Test

		ldx	#$30
		lda	#$D0
		sta	$30,X

		clc
		lda	#$30
		adc	$30,X
L231c
		bcc	L231c		; error, ->
L232c
		bvs	L232c		; error, ->
L233c
		bne	L233c		; error, ->
L234c
		bmi	L234c		; error, ->


		cmp	#0
L235c
		bne	L235c		; error, ->


		lda	#$58
		sta	Test

		lda	#$D0
		sta	TmpAB0,X

		clc
		lda	#$30
		adc	TmpAB0,X
L231d
		bcc	L231d		; error, ->
L232d
		bvs	L232d		; error, ->
L233d
		bne	L233d		; error, ->
L234d
		bmi	L234d		; error, ->


		cmp	#0
L235d
		bne	L235d		; error, ->


		lda	#$59
		sta	Test

		ldy	#$30
		lda	#$D0
		sta	TmpAB0,Y

		clc
		lda	#$30
		adc	TmpAB0,Y
L231e
		bcc	L231e		; error, ->
L232e
		bvs	L232e		; error, ->
L233e
		bne	L233e		; error, ->
L234e
		bmi	L234e		; error, ->


		cmp	#0
L235e
		bne	L235e		; error, ->


		lda	#$5A
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		lda	#$D0
		sta	($20,X)

		clc
		lda	#$30
		adc	($20,X)
L231f
		bcc	L231f		; error, ->
L232f
		bvs	L232f		; error, ->
L233f
		bne	L233f		; error, ->
L234f
		bmi	L234f		; error, ->


		cmp	#0
L235f
		bne	L235f		; error, ->


		lda	#$5B
		sta	Test

		lda	#$D0
		sta	($F0,X)

		clc
		lda	#$30
		adc	($F0,X)
L231g
		bcc	L231g		; error, ->
L232g
		bvs	L232g		; error, ->
L233g
		bne	L233g		; error, ->
L234g
		bmi	L234g		; error, ->


		cmp	#0
L235g
		bne	L235g		; error, ->


;  CLC 		; $30 + $50 = $80, returns C = 0, returns V = 1
;  LDA #$30
;  ADC #$50

		lda	#$5C
		sta	Test

		lda	#$50
		sta	TmpZP0
		sta	TmpAB0

		clc
		lda	#$30
		adc	#$50
L241
		bcs	L241		; error, ->
L242
		bvc	L242		; error, ->
L243
		beq	L243		; error, ->
L244
		bpl	L244		; error, ->


		cmp	#$80
L245
		bne	L245		; error, ->


		lda	#$5D
		sta	Test

		clc
		lda	#$30
		adc	TmpZP0
L241a
		bcs	L241a		; error, ->
L242a
		bvc	L242a		; error, ->
L243a
		beq	L243a		; error, ->
L244a
		bpl	L244a		; error, ->


		cmp	#$80
L245a
		bne	L245a		; error, ->


		lda	#$5E
		sta	Test

		clc
		lda	#$30
		adc	TmpAB0
L241b
		bcs	L241b		; error, ->
L242b
		bvc	L242b		; error, ->
L243b
		beq	L243b		; error, ->
L244b
		bpl	L244b		; error, ->


		cmp	#$80
L245b
		bne	L245b		; error, ->


		lda	#$5F
		sta	Test

		ldx	#$30
		lda	#$50
		sta	$30,X

		clc
		lda	#$30
		adc	$30,X
L241c
		bcs	L241c		; error, ->
L242c
		bvc	L242c		; error, ->
L243c
		beq	L243c		; error, ->
L244c
		bpl	L244c		; error, ->


		cmp	#$80
L245c
		bne	L245c		; error, ->


		lda	#$60
		sta	Test

		lda	#$50
		sta	TmpAB0,X

		clc
		lda	#$30
		adc	TmpAB0,X
L241d
		bcs	L241d		; error, ->
L242d
		bvc	L242d		; error, ->
L243d
		beq	L243d		; error, ->
L244d
		bpl	L244d		; error, ->


		cmp	#$80
L245d
		bne	L245d		; error, ->


		lda	#$61
		sta	Test

		ldy	#$30
		lda	#$50
		sta	TmpAB0,Y

		clc
		lda	#$30
		adc	TmpAB0,Y
L241e
		bcs	L241e		; error, ->
L242e
		bvc	L242e		; error, ->
L243e
		beq	L243e		; error, ->
L244e
		bpl	L244e		; error, ->


		cmp	#$80
L245e
		bne	L245e		; error, ->


		lda	#$62
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		lda	#$50
		sta	($20,X)

		clc
		lda	#$30
		adc	($20,X)
L241f
		bcs	L241f		; error, ->
L242f
		bvc	L242f		; error, ->
L243f
		beq	L243f		; error, ->
L244f
		bpl	L244f		; error, ->


		cmp	#$80
L245f
		bne	L245f		; error, ->


		lda	#$63
		sta	Test

		lda	#$50
		sta	($F0,X)

		clc
		lda	#$30
		adc	($F0,X)
L241g
		bcs	L241g		; error, ->
L242g
		bvc	L242g		; error, ->
L243g
		beq	L243g		; error, ->
L244g
		bpl	L244g		; error, ->


		cmp	#$80
L245g
		bne	L245g		; error, ->


;  CLC 		; $80 + $FF = $7F, returns C = 1, returns V = 1
;  LDA #$80
;  ADC #$FF

		lda	#$64
		sta	Test

		lda	#$FF
		sta	TmpZP0
		sta	TmpAB0

		clc
		lda	#$80
		adc	#$FF
L251
		bcc	L251		; error, ->
L252
		bvc	L252		; error, ->
L253
		beq	L253		; error, ->
L254
		bmi	L254		; error, ->


		cmp	#$7F
L255
		bne	L255		; error, ->


		lda	#$65
		sta	Test

		clc
		lda	#$80
		adc	TmpZP0
L251a
		bcc	L251a		; error, ->
L252a
		bvc	L252a		; error, ->
L253a
		beq	L253a		; error, ->
L254a
		bmi	L254a		; error, ->


		cmp	#$7F
L255a
		bne	L255a		; error, ->


		lda	#$66
		sta	Test

		clc
		lda	#$80
		adc	TmpAB0
L251b
		bcc	L251b		; error, ->
L252b
		bvc	L252b		; error, ->
L253b
		beq	L253b		; error, ->
L254b
		bmi	L254b		; error, ->


		cmp	#$7F
L255b
		bne	L255b		; error, ->


		lda	#$67
		sta	Test

		ldx	#$80
		lda	#$FF
		sta	$30,X

		clc
		lda	#$80
		adc	$30,X
L251c
		bcc	L251c		; error, ->
L252c
		bvc	L252c		; error, ->
L253c
		beq	L253c		; error, ->
L254c
		bmi	L254c		; error, ->


		cmp	#$7F
L255c
		bne	L255c		; error, ->


		lda	#$68
		sta	Test

		lda	#$FF
		sta	TmpAB0,X

		clc
		lda	#$80
		adc	TmpAB0,X
L251d
		bcc	L251d		; error, ->
L252d
		bvc	L252d		; error, ->
L253d
		beq	L253d		; error, ->
L254d
		bmi	L254d		; error, ->


		cmp	#$7F
L255d
		bne	L255d		; error, ->


		lda	#$69
		sta	Test

		ldy	#$80
		lda	#$FF
		sta	TmpAB0,Y

		clc
		lda	#$80
		adc	TmpAB0,Y
L251e
		bcc	L251e		; error, ->
L252e
		bvc	L252e		; error, ->
L253e
		beq	L253e		; error, ->
L254e
		bmi	L254e		; error, ->


		cmp	#$7F
L255e
		bne	L255e		; error, ->


		lda	#$6A
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		lda	#$FF
		sta	($20,X)

		clc
		lda	#$80
		adc	($20,X)
L251f
		bcc	L251f		; error, ->
L252f
		bvc	L252f		; error, ->
L253f
		beq	L253f		; error, ->
L254f
		bmi	L254f		; error, ->


		cmp	#$7F
L255f
		bne	L255f		; error, ->


		lda	#$6B
		sta	Test

		lda	#$FF
		sta	($F0,X)

		clc
		lda	#$80
		adc	($F0,X)
L251g
		bcc	L251g		; error, ->
L252g
		bvc	L252g		; error, ->
L253g
		beq	L253g		; error, ->
L254g
		bmi	L254g		; error, ->


		cmp	#$7F
L255g
		bne	L255g		; error, ->


;**  SBC  (1)
;  SEC 		; $30 - $30 = $00, returns C = 1, returns V = 0
;  LDA #$30
;  SBC #$30

		lda	#$6C
		sta	Test

		lda	#$30
		sta	TmpZP0
		sta	TmpAB0

		sec
		sbc	#$30
L261
		bcc	L261		; error, ->
L262
		bvs	L262		; error, ->
L263
		bne	L263		; error, ->
L264
		bmi	L264		; error, ->


		cmp	#0
L265
		bne	L265		; error, ->


		lda	#$6D
		sta	Test

		sec
		lda	#$30
		sbc	TmpZP0
L261a
		bcc	L261a		; error, ->
L262a
		bvs	L262a		; error, ->
L263a
		bne	L263a		; error, ->
L264a
		bmi	L264a		; error, ->


		cmp	#0
L265a
		bne	L265a		; error, ->


		lda	#$6E
		sta	Test

		sec
		lda	#$30
		sbc	TmpAB0
L261b
		bcc	L261b		; error, ->
L262b
		bvs	L262b		; error, ->
L263b
		bne	L263b		; error, ->
L264b
		bmi	L264b		; error, ->


		cmp	#0
L265b
		bne	L265b		; error, ->


		lda	#$6F
		sta	Test

		lda	#$30
		tax
		sta	$30,X

		sec
		sbc	$30,X
L261c
		bcc	L261c		; error, ->
L262c
		bvs	L262c		; error, ->
L263c
		bne	L263c		; error, ->
L264c
		bmi	L264c		; error, ->


		cmp	#0
L265c
		bne	L265c		; error, ->


		lda	#$70
		sta	Test

		lda	#$30
		sta	TmpAB0,X

		sec
		sbc	TmpAB0,X
L261d
		bcc	L261d		; error, ->
L262d
		bvs	L262d		; error, ->
L263d
		bne	L263d		; error, ->
L264d
		bmi	L264d		; error, ->


		cmp	#0
L265d
		bne	L265d		; error, ->


		lda	#$71
		sta	Test

		lda	#$30
		tay
		sta	TmpAB0,Y

		sec
		sbc	TmpAB0,Y
L261e
		bcc	L261e		; error, ->
L262e
		bvs	L262e		; error, ->
L263e
		bne	L263e		; error, ->
L264e
		bmi	L264e		; error, ->


		cmp	#0
L265e
		bne	L265e		; error, ->


		lda	#$72
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		ldx	#$30
		txa
		sta	($20,X)

		sec
		sbc	($20,X)
L261f
		bcc	L261f		; error, ->
L262f
		bvs	L262f		; error, ->
L263f
		bne	L263f		; error, ->
L264f
		bmi	L264f		; error, ->


		cmp	#0
L265f
		bne	L265f		; error, ->


		lda	#$73
		sta	Test

		lda	#$30
		sta	($F0,X)

		sec
		sbc	($F0,X)
L261g
		bcc	L261g		; error, ->
L262g
		bvs	L262g		; error, ->
L263g
		bne	L263g		; error, ->
L264g
		bmi	L264g		; error, ->


		cmp	#0
L265g
		bne	L265g		; error, ->


;  SEC 		; $30 - $50 = $20, returns C = 0, returns V = 0
;  LDA #$30
;  SBC #$50

		lda	#$74
		sta	Test

		lda	#$50
		sta	TmpZP0
		sta	TmpAB0

		sec
		lda	#$30
		sbc	#$50
L271
		bcs	L271		; error, ->
L272
		bvs	L272		; error, ->
L273
		beq	L273		; error, ->
L274
		bpl	L274		; error, ->


		cmp	#$E0
L275
		bne	L275		; error, ->


		lda	#$75
		sta	Test

		sec
		lda	#$30
		sbc	TmpZP0
L271a
		bcs	L271a		; error, ->
L272a
		bvs	L272a		; error, ->
L273a
		beq	L273a		; error, ->
L274a
		bpl	L274a		; error, ->


		cmp	#$E0
L275a
		bne	L275a		; error, ->

		lda	#$76
		sta	Test

		sec
		lda	#$30
		sbc	TmpAB0
L271b
		bcs	L271b		; error, ->
L272b
		bvs	L272b		; error, ->
L273b
		beq	L273b		; error, ->
L274b
		bpl	L274b		; error, ->


		cmp	#$E0
L275b
		bne	L275b		; error, ->

		lda	#$77
		sta	Test

		ldx	#$30
		lda	#$50
		sta	$30,X

		sec
		lda	#$30
		sbc	$30,X
L271c
		bcs	L271c		; error, ->
L272c
		bvs	L272c		; error, ->
L273c
		beq	L273c		; error, ->
L274c
		bpl	L274c		; error, ->


		cmp	#$E0
L275c
		bne	L275c		; error, ->

		lda	#$78
		sta	Test

		lda	#$50
		sta	TmpAB0,X

		sec
		lda	#$30
		sbc	TmpAB0,X
L271d
		bcs	L271d		; error, ->
L272d
		bvs	L272d		; error, ->
L273d
		beq	L273d		; error, ->
L274d
		bpl	L274d		; error, ->


		cmp	#$E0
L275d
		bne	L275d		; error, ->

		lda	#$79
		sta	Test

		ldy	#$30
		lda	#$50
		sta	TmpAB0,Y

		sec
		lda	#$30
		sbc	TmpAB0,Y
L271e
		bcs	L271e		; error, ->
L272e
		bvs	L272e		; error, ->
L273e
		beq	L273e		; error, ->
L274e
		bpl	L274e		; error, ->


		cmp	#$E0
L275e
		bne	L275e		; error, ->


		lda	#$7A
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		lda	#$50
		sta	($20,X)

		sec
		lda	#$30
		sbc	($20,X)
L271f
		bcs	L271f		; error, ->
L272f
		bvs	L272f		; error, ->
L273f
		beq	L273f		; error, ->
L274f
		bpl	L274f		; error, ->


		cmp	#$E0
L275f
		bne	L275f		; error, ->


		lda	#$7B
		sta	Test

		lda	#$50
		sta	($F0,X)

		sec
		lda	#$30
		sbc	($F0,X)
L271g
		bcs	L271g		; error, ->
L272g
		bvs	L272g		; error, ->
L273g
		beq	L273g		; error, ->
L274g
		bpl	L274g		; error, ->


		cmp	#$E0
L275g
		bne	L275g		; error, ->


;  SEC 		; $80 - $01 = $7F, returns C = 1, returns V = 1
;  LDA #$80
;  SBC #$01

		lda	#$7C
		sta	Test

		lda	#1
		sta	TmpZP0
		sta	TmpAB0

		sec
		lda	#$80
		sbc	#1
L281
		bcc	L281		; error, ->
L282
		bvc	L282		; error, ->
L283
		beq	L283		; error, ->
L284
		bmi	L284		; error, ->


		cmp	#$7F
L285
		bne	L285		; error, ->


		lda	#$7D
		sta	Test

		sec
		lda	#$80
		sbc	TmpZP0
L281a
		bcc	L281a		; error, ->
L282a
		bvc	L282a		; error, ->
L283a
		beq	L283a		; error, ->
L284a
		bmi	L284a		; error, ->


		cmp	#$7F
L285a
		bne	L285a		; error, ->


		lda	#$7E
		sta	Test

		sec
		lda	#$80
		sbc	TmpAB0
L281b
		bcc	L281b		; error, ->
L282b
		bvc	L282b		; error, ->
L283b
		beq	L283b		; error, ->
L284b
		bmi	L284b		; error, ->


		cmp	#$7F
L285b
		bne	L285b		; error, ->


		lda	#$7F
		sta	Test

		ldx	#$30
		lda	#1
		sta	$30,X

		sec
		lda	#$80
		sbc	$30,X
L281c
		bcc	L281c		; error, ->
L282c
		bvc	L282c		; error, ->
L283c
		beq	L283c		; error, ->
L284c
		bmi	L284c		; error, ->


		cmp	#$7F
L285c
		bne	L285c		; error, ->


		lda	#$80
		sta	Test

		lda	#1
		sta	TmpAB0,X

		sec
		lda	#$80
		sbc	TmpAB0,X
L281d
		bcc	L281d		; error, ->
L282d
		bvc	L282d		; error, ->
L283d
		beq	L283d		; error, ->
L284d
		bmi	L284d		; error, ->


		cmp	#$7F
L285d
		bne	L285d		; error, ->


		lda	#$81
		sta	Test

		ldy	#$30
		lda	#1
		sta	TmpAB0,Y

		sec
		lda	#$80
		sbc	TmpAB0,Y
L281e
		bcc	L281e		; error, ->
L282e
		bvc	L282e		; error, ->
L283e
		beq	L283e		; error, ->
L284e
		bmi	L284e		; error, ->


		cmp	#$7F
L285e
		bne	L285e		; error, ->


		lda	#$82
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		lda	#1
		sta	($20,X)

		sec
		lda	#$80
		sbc	($20,X)
L281f
		bcc	L281f		; error, ->
L282f
		bvc	L282f		; error, ->
L283f
		beq	L283f		; error, ->
L284f
		bmi	L284f		; error, ->


		cmp	#$7F
L285f
		bne	L285f		; error, ->


		lda	#$83
		sta	Test

		lda	#1
		sta	($F0,X)

		sec
		lda	#$80
		sbc	($F0,X)
L281g
		bcc	L281g		; error, ->
L282g
		bvc	L282g		; error, ->
L283g
		beq	L283g		; error, ->
L284g
		bmi	L284g		; error, ->


		cmp	#$7F
L285g
		bne	L285g		; error, ->


;  SEC 		; $70 - $E0 = $90, returns C = 0, returns V = 1
;  LDA #$70
;  SBC #$E0

		lda	#$84
		sta	Test

		lda	#$E0
		sta	TmpZP0
		sta	TmpAB0

		sec
		lda	#$70
		sbc	#$E0
L291
		bcs	L291		; error, ->
L292
		bvc	L292		; error, ->
L293
		beq	L293		; error, ->
L294
		bpl	L294		; error, ->


		cmp	#$90
L295
		bne	L295		; error, ->


		lda	#$85
		sta	Test

		sec
		lda	#$70
		sbc	TmpZP0
L291a
		bcs	L291a		; error, ->
L292a
		bvc	L292a		; error, ->
L293a
		beq	L293a		; error, ->
L294a
		bpl	L294a		; error, ->


		cmp	#$90
L295a
		bne	L295a		; error, ->


		lda	#$86
		sta	Test

		sec
		lda	#$70
		sbc	TmpAB0
L291b
		bcs	L291b		; error, ->
L292b
		bvc	L292b		; error, ->
L293b
		beq	L293b		; error, ->
L294b
		bpl	L294b		; error, ->


		cmp	#$90
L295b
		bne	L295b		; error, ->


		lda	#$87
		sta	Test

		ldx	#$70
		lda	#$E0
		sta	$30,X

		sec
		lda	#$70
		sbc	$30,X
L291c
		bcs	L291c		; error, ->
L292c
		bvc	L292c		; error, ->
L293c
		beq	L293c		; error, ->
L294c
		bpl	L294c		; error, ->


		cmp	#$90
L295c
		bne	L295c		; error, ->


		lda	#$88
		sta	Test

		lda	#$E0
		sta	TmpAB0,X

		sec
		lda	#$70
		sbc	TmpAB0,X
L291d
		bcs	L291d		; error, ->
L292d
		bvc	L292d		; error, ->
L293d
		beq	L293d		; error, ->
L294d
		bpl	L294d		; error, ->


		cmp	#$90
L295d
		bne	L295d		; error, ->


		lda	#$89
		sta	Test

		ldy	#$70
		lda	#$E0
		sta	TmpAB0,Y

		sec
		lda	#$70
		sbc	TmpAB0,Y
L291e
		bcs	L291e		; error, ->
L292e
		bvc	L292e		; error, ->
L293e
		beq	L293e		; error, ->
L294e
		bpl	L294e		; error, ->


		cmp	#$90
L295e
		bne	L295e		; error, ->


		lda	#$8A
		sta	Test

		lda	#$10
		sta	$51
		sta	$21

		lda	#$AA
		sta	$50
		sta	$20

		lda	#$E0
		sta	($20,X)

		sec
		lda	#$70
		sbc	($20,X)
L291f
		bcs	L291f		; error, ->
L292f
		bvc	L292f		; error, ->
L293f
		beq	L293f		; error, ->
L294f
		bpl	L294f		; error, ->


		cmp	#$90
L295f
		bne	L295f		; error, ->


		lda	#$8B
		sta	Test

		lda	#$E0
		sta	($F0,X)

		sec
		lda	#$70
		sbc	($F0,X)
L291g
		bcs	L291g		; error, ->
L292g
		bvc	L292g		; error, ->
L293g
		beq	L293g		; error, ->
L294g
		bpl	L294g		; error, ->


		cmp	#$90
L295g
		bne	L295g		; error, ->


;**  ADC  (2)             	Test ADC in combination with SEC
		lda	#$8C
		sta	Test

		sec
		lda	#1
		adc	#4
L301
		bcs	L301		; Carry must have been cleared

		cmp	#6
L302
		bne	L302

		lda	#$8D
		sta	Test

		sec
		lda	#$7F
		adc	#$80
L303
		bne	L303
L304
		bcc	L302		; Carry still must be set

;**  SBC  (2)             	Test SBC in combination with CLC
		lda	#$8E
		sta	Test

		clc
		lda	#6
		sbc	#4
L311
		bcc	L311		; Carry must have been set

		cmp	#1
L312
		bne	L312

		lda	#$8F
		sta	Test

		sec
		lda	#$7F
		adc	#$7F
L313
		bcs	L313		; Carry still must be cleared

		cmp	#$FF
L314
		bne	L314

		lda	#$9F
		sta	Test

		sec
		lda	#$7F
		adc	#$80
L315
		bcc	L313		; Carry still must be set
L316
		bne	L316


;**  AND
		lda	#$A0
		sta	Test

		ldx	#5
		ldy	#9

		lda	#$10
		sta	TmpZP0+5
		sta	TmpZP0+6

		lda	#0
		sta	TmpZP0+4
		sta	TmpAB0+$10
		sta	TmpAB0+9

		lda	#$AA
		sta	(TmpZP0,X)		; = ($15) = $1010
		sta	(TmpZP0+4),Y		; = $1000 + 9

		cmp	TmpAB0+$10
L320a
		bne	L320a

		cmp	TmpAB0+9

L320b		bne	L320b


;* A = $FF, memory = 0
		lda	#$A1
		sta	Test

		lda	#0
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#$FF
		and	#$0
L321
		bne	L321		; error, ->
L322
		bmi	L322		; error, ->

		lda	#$A2
		sta	Test

		lda	#$FF
		and	TmpZP0
L331
		bne	L331		; error, ->
L332
		bmi	L332		; error, ->

		lda	#$A3
		sta	Test

		lda	#$FF
		and	TmpAB0
L341
		bne	L341		; error, ->
L342
		bmi	L342		; error, ->

		lda	#$A4
		sta	Test

		lda	#$FF
		and	TmpZP0,X
L351
		bne	L351		; error, ->
L352
		bmi	L352		; error, ->

		lda	#$A5
		sta	Test

		lda	#$FF
		and	TmpAB0,X
L361
		bne	L361		; error, ->
L362
		bmi	L362		; error, ->

		lda	#$A6
		sta	Test

		lda	#$FF
		and	TmpAB0,Y
L371
		bne	L371		; error, ->
L372
		bmi	L372		; error, ->

		lda	#$A7
		sta	Test

		lda	#$FF
		and	(TmpZP0,X)
L381
		bne	L381		; error, ->
L382
		bmi	L382		; error, ->

		lda	#$A8
		sta	Test

		lda	#$FF
		and	(TmpZP0+4),Y
L391
		bne	L391		; error, ->
L392
		bmi	L392		; error, ->

;* A = 0, memory = $FF
		lda	#$A9
		sta	Test

		lda	#$FF
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#0
		and	#$FF
L421
		bne	L421		; error, ->
L422
		bmi	L422		; error, ->

		lda	#$AA
		sta	Test

		lda	#0
		and	TmpZP0
L431
		bne	L431		; error, ->
L432
		bmi	L432		; error, ->

		lda	#$AB
		sta	Test

		lda	#0
		and	TmpAB0
L441
		bne	L441		; error, ->
L442
		bmi	L442		; error, ->

		lda	#$AC
		sta	Test

		lda	#0
		and	TmpZP0,X
L451
		bne	L451		; error, ->
L452
		bmi	L452		; error, ->

		lda	#$AD
		sta	Test

		lda	#0
		and	TmpAB0,X
L461
		bne	L461		; error, ->
L462
		bmi	L462		; error, ->

		lda	#$AE
		sta	Test

		lda	#0
		and	TmpAB0,Y
L471
		bne	L471		; error, ->
L472
		bmi	L472		; error, ->

		lda	#$AF
		sta	Test

		lda	#0
		and	(TmpZP0,X)
L481
		bne	L481		; error, ->
L482
		bmi	L482		; error, ->

		lda	#$B0
		sta	Test

		lda	#0
		and	(TmpZP0+4),Y
L491
		bne	L491		; error, ->
L492
		bmi	L492		; error, ->

;* A = $AA, memory = $55
		lda	#$B1
		sta	Test

		lda	#$55
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#$AA
		and	#$55
L521
		bne	L521		; error, ->
L522
		bmi	L522		; error, ->

		lda	#$B2
		sta	Test

		lda	#$AA
		and	TmpZP0
L531
		bne	L531		; error, ->
L532
		bmi	L532		; error, ->

		lda	#$B3
		sta	Test

		lda	#$AA
		and	TmpAB0
L541
		bne	L541		; error, ->
L542
		bmi	L542		; error, ->

		lda	#$B4
		sta	Test

		lda	#$AA
		and	TmpZP0,X
L551
		bne	L551		; error, ->
L552
		bmi	L552		; error, ->

		lda	#$B5
		sta	Test

		lda	#$AA
		and	TmpAB0,X
L561
		bne	L561		; error, ->
L562
		bmi	L562		; error, ->

		lda	#$B6
		sta	Test

		lda	#$AA
		and	TmpAB0,Y
L571
		bne	L571		; error, ->
L572
		bmi	L572		; error, ->

		lda	#$B7
		sta	Test

		lda	#$AA
		and	(TmpZP0,X)
L581
		bne	L581		; error, ->
L582
		bmi	L582		; error, ->

		lda	#$B8
		sta	Test

		lda	#$AA
		and	(TmpZP0+4),Y
L591
		bne	L591		; error, ->
L592
		bmi	L592		; error, ->

;* A = $FF, memory = $FF
		lda	#$B9
		sta	Test

		lda	#$FF
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#$FF
		and	#$FF
L622
		bpl	L622		; error, ->

		cmp	#$FF
L621
		bne	L621		; error, ->

		lda	#$BA
		sta	Test

		lda	#$FF
		and	TmpZP0
L632
		bpl	L632		; error, ->

		cmp	#$FF
L631
		bne	L631		; error, ->

		lda	#$BB
		sta	Test

		lda	#$FF
		and	TmpAB0
L642
		bpl	L642		; error, ->

		cmp	#$FF
L641
		bne	L641		; error, ->

		lda	#$BC
		sta	Test

		lda	#$FF
		and	TmpZP0,X
L652
		bpl	L652		; error, ->

		cmp	#$FF
L651
		bne	L651		; error, ->

		lda	#$BD
		sta	Test

		lda	#$FF
		and	TmpAB0,X
L662
		bpl	L662		; error, ->

		cmp	#$FF
L661
		bne	L661		; error, ->

		lda	#$BE
		sta	Test

		lda	#$FF
		and	TmpAB0,Y
L672
		bpl	L672		; error, ->

		cmp	#$FF
L671
		bne	L671		; error, ->

		lda	#$BF
		sta	Test

		lda	#$FF
		and	(TmpZP0,X)
L682
		bpl	L682		; error, ->

		cmp	#$FF
L681
		bne	L681		; error, ->

		lda	#$C0
		sta	Test

		lda	#$FF
		and	(TmpZP0+4),Y
L692
		bpl	L692		; error, ->

		cmp	#$FF
L691
		bne	L691		; error, ->


;**  ORA
;* A = $FF, memory = 0
		lda	#$C1
		sta	Test

		lda	#0
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#$FF
		ora	#0
L722
		bpl	L722		; error, ->

		cmp	#$FF
L721
		bne	L721		; error, ->

		lda	#$C2
		sta	Test

		lda	#$FF
		ora	TmpZP0
L732
		bpl	L732		; error, ->

		cmp	#$FF
L731
		bne	L731		; error, ->

		lda	#$C3
		sta	Test

		lda	#$FF
		ora	TmpAB0
L742
		bpl	L742		; error, ->

		cmp	#$FF
L741
		bne	L741		; error, ->

		lda	#$C4
		sta	Test

		lda	#$FF
		ora	TmpZP0,X
L752
		bpl	L752		; error, ->

		cmp	#$FF
L751
		bne	L751		; error, ->

		lda	#$C5
		sta	Test

		lda	#$FF
		ora	TmpAB0,X
L762
		bpl	L762		; error, ->

		cmp	#$FF
L761
		bne	L761		; error, ->

		lda	#$C6
		sta	Test

		lda	#$FF
		ora	TmpAB0,Y
L772
		bpl	L772		; error, ->

		cmp	#$FF
L771
		bne	L771		; error, ->

		lda	#$C7
		sta	Test

		lda	#$FF
		ora	(TmpZP0,X)
L782
		bpl	L782		; error, ->

		cmp	#$FF
L781
		bne	L781		; error, ->

		lda	#$C8
		sta	Test

		lda	#$FF
		ora	(TmpZP0+4),Y
L792
		bpl	L792		; error, ->

		cmp	#$FF
L791
		bne	L791		; error, ->

;* A = 0, memory = $FF
		lda	#$C9
		sta	Test

		lda	#$FF
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#0
		ora	#$FF
L822
		bpl	L822		; error, ->

		cmp	#$FF
L821
		bne	L821		; error, ->

		lda	#$CA
		sta	Test

		lda	#0
		ora	TmpZP0
L832
		bpl	L832		; error, ->

		cmp	#$FF
L831
		bne	L831		; error, ->

		lda	#$CB
		sta	Test

		lda	#0
		ora	TmpAB0
L842
		bpl	L842		; error, ->

		cmp	#$FF
L841
		bne	L841		; error, ->

		lda	#$CC
		sta	Test

		lda	#0
		ora	TmpZP0,X
L852
		bpl	L852		; error, ->

		cmp	#$FF
L851
		bne	L851		; error, ->

		lda	#$CD
		sta	Test

		lda	#0
		ora	TmpAB0,X
L862
		bpl	L862		; error, ->

		cmp	#$FF
L861
		bne	L861		; error, ->

		lda	#$CE
		sta	Test

		lda	#0
		ora	TmpAB0,Y
L872
		bpl	L872		; error, ->

		cmp	#$FF
L871
		bne	L871		; error, ->

		lda	#$CF
		sta	Test

		lda	#0
		ora	(TmpZP0,X)
L882
		bpl	L882		; error, ->

		cmp	#$FF
L881
		bne	L881		; error, ->

		lda	#$D0
		sta	Test

		lda	#0
		ora	(TmpZP0+4),Y
L892
		bpl	L892		; error, ->

		cmp	#$FF
L891
		bne	L891		; error, ->

;* A = 0, memory = 0
		lda	#$D1
		sta	Test

		lda	#0
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#0
		ora	#0
L922
		bmi	L922		; error, ->
L921
		bne	L921		; error, ->

		lda	#$CA
		sta	Test

		lda	#0
		ora	TmpZP0
L932
		bmi	L932		; error, ->
L931
		bne	L931		; error, ->

		lda	#$CB
		sta	Test

		lda	#0
		ora	TmpAB0
L942
		bmi	L942		; error, ->
L941
		bne	L941		; error, ->

		lda	#$CC
		sta	Test

		lda	#0
		ora	TmpZP0,X
L952
		bmi	L952		; error, ->
L951
		bne	L951		; error, ->

		lda	#$CD
		sta	Test

		lda	#0
		ora	TmpAB0,X
L962
		bmi	L962		; error, ->
L961
		bne	L961		; error, ->

		lda	#$CE
		sta	Test

		lda	#0
		ora	TmpAB0,Y
L972
		bmi	L972		; error, ->
L971
		bne	L971		; error, ->

		lda	#$CF
		sta	Test

		lda	#0
		ora	(TmpZP0,X)
L982
		bmi	L982		; error, ->
L981
		bne	L981		; error, ->

		lda	#$D0
		sta	Test

		lda	#0
		ora	(TmpZP0+4),Y
L992
		bmi	L992		; error, ->
L991
		bne	L991		; error, ->


;**  ASL
;* ASL A
		lda	#$D1
		sta	Test

		lda	#1
		sec
		asl
M001
		bcs	M001
M002
		bmi	M003

		cmp	#2
M003
		bne	M003

		sec
		asl
M004
		bcs	M004
M005
		bmi	M005

		cmp	#4
M006
		bne	M006

		sec
		asl
M007
		bcs	M007
M008
		bmi	M008

		cmp	#8
M009
		bne	M009

		sec
		asl
M010
		bcs	M010
M011
		bmi	M011

		cmp	#$10
M012
		bne	M012

		sec
		asl
M013
		bcs	M013
M014
		bmi	M014

		cmp	#$20
M015
		bne	M015

		sec
		asl
M016
		bcs	M016
M017
		bmi	M017

		cmp	#$40
M018
		bne	M018

		sec
		asl
M019
		bcs	M019
M020
		bpl	M020

		cmp	#$80
M021
		bne	M021

		asl
M022
		bcc	M022
M023
		bmi	M023
M024
		bne	M024

;* Rest of possible ASL opcodes. Only read/write has to be tested because
;   the shifting itself doesn't have to be tested anymore.  
		lda	#$55
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X

		lda	#$D2
		sta	Test

		asl	TmpZP0
		lda	TmpZP0
		cmp	#$AA
M025		
		bne	M025

		lda	#$D3
		sta	Test

		asl	TmpAB0
		lda	TmpAB0
		cmp	#$AA
M026		
		bne	M026

		lda	#$D4
		sta	Test

		asl	TmpZP0,X
		lda	TmpZP0,X
		cmp	#$AA
M027		
		bne	M027

		lda	#$D5
		sta	Test

		asl	TmpAB0,X
		lda	TmpAB0,X
		cmp	#$AA
M028		
		bne	M028


;**  LSR
		lda	#$D6
		sta	Test

		lda	#$80
		sec
		lsr
M031
		bcs	M031
M032
		bmi	M032

		cmp	#$40
M033
		bne	M033

		sec
		lsr
M034
		bcs	M034
M035
		bmi	M035

		cmp	#$20
M036
		bne	M036

		sec
		lsr
M037
		bcs	M037
M038
		bmi	M038

		cmp	#$10
M039
		bne	M039

		sec
		lsr
M041
		bcs	M041
M042
		bmi	M043

		cmp	#8
M043
		bne	M043

		sec
		lsr
M044
		bcs	M044
M045
		bmi	M045

		cmp	#4
M046
		bne	M046

		sec
		lsr
M047
		bcs	M047
M048
		bmi	M048

		cmp	#2
M049
		bne	M049

		sec
		lsr
M051
		bcs	M051
M052
		bmi	M053

		cmp	#1
M053
		bne	M053

		lsr
M054
		bcc	M054
M055
		bmi	M055
M056
		bne	M056

;* Rest of possible LSR opcodes. Only read/write has to be tested because
;   the shifting itself doesn't have to be tested anymore.  
		lda	#$AA
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X

		lda	#$D7
		sta	Test

		lsr	TmpZP0
		lda	TmpZP0
		cmp	#$55
M057		
		bne	M057

		lda	#$D8
		sta	Test

		lsr	TmpAB0
		lda	TmpAB0
		cmp	#$55
M058		
		bne	M058

		lda	#$D9
		sta	Test

		lsr	TmpZP0,X
		lda	TmpZP0,X
		cmp	#$55
M059		
		bne	M059

		lda	#$DA
		sta	Test

		lsr	TmpAB0,X
		lda	TmpAB0,X
		cmp	#$55
M060		
		bne	M060


;**  ROL
		lda	#$D8
		sta	Test

		lda	#$11
		clc
		rol
M071
		bcs	M071
M072
		bmi	M072
		
		cmp	#$22
M073
		bne	M073
		
		sec
		rol
M074
		bcs	M074
M075
		bmi	M075
		
		cmp	#$45
M076
		bne	M076
		
		clc
		rol
M077
		bcs	M077
M078
		bpl	M078
		
		cmp	#$8A
M079
		bne	M079
		
		clc
		rol
M081
		bcc	M081
M082
		bmi	M082
		
		cmp	#TmpZP0+4
M083
		bne	M083
		
;* Rest of possible ROL opcodes. Only read/write has to be tested because
;   the shifting itself doesn't have to be tested anymore.  
		lda	#$55
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X

		lda	#$D9
		sta	Test

		sec
		rol	TmpZP0
		lda	TmpZP0
		cmp	#$AB
M085		
		bne	M085

		lda	#$DA
		sta	Test

		sec
		rol	TmpAB0
		lda	TmpAB0
		cmp	#$AB
M086		
		bne	M086

		lda	#$DB
		sta	Test

		sec
		rol	TmpZP0,X
		lda	TmpZP0,X
		cmp	#$AB
M087		
		bne	M087

		lda	#$DC
		sta	Test

		sec
		rol	TmpAB0,X
		lda	TmpAB0,X
		cmp	#$AB
M088		
		bne	M088

;**  ROR
		lda	#$DD
		sta	Test

		lda	#$88
		clc
		ror
M091
		bcs	M091
M092
		bmi	M092
		
		cmp	#$44
M093
		bne	M093
		
		sec
		ror
M094
		bcs	M094
M095
		bpl	M095
		
		cmp	#$A2
M096
		bne	M096
		
		clc
		ror
M097
		bcs	M097
M098
		bmi	M098
		
		cmp	#$51
M099
		bne	M099
		
		clc
		ror
M101
		bcc	M101
M102
		bmi	M102
		
		cmp	#$28
M103
		bne	M103
		
;* Rest of possible ror opcodes. Only read/write has to be tested because
;   the shifting itself doesn't have to be tested anymore.  
		lda	#$AA
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X

		lda	#$DE
		sta	Test

		sec
		ror	TmpZP0
		lda	TmpZP0
		cmp	#$D5
M105		
		bne	M105

		lda	#$DF
		sta	Test

		sec
		ror	TmpAB0
		lda	TmpAB0
		cmp	#$D5
M106		
		bne	M106

		lda	#$E0
		sta	Test

		sec
		ror	TmpZP0,X
		lda	TmpZP0,X
		cmp	#$D5
M107		
		bne	M107

		lda	#$E1
		sta	Test

		sec
		ror	TmpAB0,X
		lda	TmpAB0,X
		cmp	#$D5
M108		
		bne	M108


;**  BIT
		lda	#$E1
		sta	Test

;* A = $FF, memory = 0
		lda	#0
		sta	TmpZP0
		sta	TmpAB0

		lda	#$FF
		bit	TmpZP0
M111
		bne	M111		; error, ->
M112
		bmi	M112		; error, ->
M113
		bvs	M113		; error, ->

		lda	#$E2
		sta	Test

		lda	#$FF
		bit	TmpAB0
M121
		bne	M121		; error, ->
M122
		bmi	M122		; error, ->
M123
		bvs	M123		; error, ->


;* A = 0, memory = $FF
		lda	#$E3
		sta	Test

		lda	#$FF
		sta	TmpZP0
		sta	TmpAB0

		lda	#0
		bit	TmpZP0
M131
		bne	M131		; error, ->
M132
		bpl	M132		; error, ->
M133
		bvc	M133		; error, ->

		lda	#$E4
		sta	Test

		lda	#0
		bit	TmpAB0
M141
		bne	M141		; error, ->
M142
		bpl	M142		; error, ->
M143
		bvc	M143		; error, ->


;* A = $54, memory = $55
		lda	#$E5
		sta	Test

		lda	#$55
		sta	TmpZP0
		sta	TmpAB0

		lda	#$54
		bit	TmpZP0
M151
		beq	M151		; error, ->
M152
		bmi	M152		; error, ->
M153
		bvc	M153		; error, ->

		lda	#$E6
		sta	Test

		lda	#$54
		bit	TmpAB0
M161
		beq	M161		; error, ->
M162
		bmi	M162		; error, ->
M163
		bvc	M163		; error, ->


;**  DEC
;   Only read/write has to be tested because the decrement itself
;   doesn't have to be tested anymore; done with DEX and DEY. 
		lda	#$E7
		sta	Test

		lda	#$AA
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X

		dec	TmpZP0
		lda	TmpZP0
		cmp	#$A9
M171
		bne	M171
		
		dec	TmpAB0
		lda	TmpAB0
		cmp	#$A9
M172
		bne	M172
		
		dec	TmpZP0,X
		lda	TmpZP0,X
		cmp	#$A9
M173
		bne	M173
		
		dec	TmpAB0,X
		lda	TmpAB0,x
		cmp	#$A9
M174
		bne	M174
		
;**  DEC
;   Only read/write has to be tested because the decrement itself
;   doesn't have to be tested anymore; done with DEX and DEY. 
		lda	#$E7
		sta	Test

		inc	TmpZP0
		lda	TmpZP0
		cmp	#$AA
M181
		bne	M181
		
		inc	TmpAB0
		lda	TmpAB0
		cmp	#$AA
M182
		bne	M182
		
		inc	TmpZP0,X
		lda	TmpZP0,X
		cmp	#$AA
M183
		bne	M183
		
		inc	TmpAB0,X
		lda	TmpAB0,x
		cmp	#$AA
M184
		bne	M184
		
		
;**  EOR
;* A = $AA, memory = $55
		lda	#$E8
		sta	Test

		lda	#$55
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#$AA
		eor	#$55
M191
		beq	M191		; error, ->
M192
		bpl	M192		; error, ->
		
		cmp	#$FF
M193
		bne	M193

		lda	#$E9
		sta	Test

		lda	#$AA
		eor	TmpZP0
M201
		beq	M201		; error, ->
M202
		bpl	M202		; error, ->

		cmp	#$FF
M203
		bne	M203

		lda	#$EA
		sta	Test

		lda	#$AA
		eor	TmpAB0
M211
		beq	M211		; error, ->
M212
		bpl	M212		; error, ->

		cmp	#$FF
M213
		bne	M213

		lda	#$EB
		sta	Test

		lda	#$AA
		eor	TmpZP0,X
M221
		beq	M221		; error, ->
M222
		bpl	M222		; error, ->

		cmp	#$FF
M223
		bne	M223

		lda	#$EC
		sta	Test

		lda	#$AA
		eor	TmpAB0,X
M231
		beq	M231		; error, ->
M232
		bpl	M232		; error, ->

		cmp	#$FF
M233
		bne	M233

		lda	#$ED
		sta	Test

		lda	#$AA
		eor	TmpAB0,Y
M241
		beq	M241		; error, ->
M242
		bpl	M242		; error, ->

		cmp	#$FF
M243
		bne	M243

		lda	#$EE
		sta	Test

		lda	#$AA
		eor	(TmpZP0,X)
M251
		beq	M251		; error, ->
M252
		bpl	M252		; error, ->

		cmp	#$FF
M253
		bne	M253

		lda	#$EF
		sta	Test

		lda	#$AA
		eor	(TmpZP0+4),Y
M261
		beq	M261		; error, ->
M262
		bpl	M262		; error, ->

		cmp	#$FF
M263
		bne	M263

;* A = $CC, memory = $CC
		lda	#$F0
		sta	Test

		lda	#$CC
		sta	TmpZP0
		sta	TmpAB0
		sta	TmpZP0,X
		sta	TmpAB0,X
		sta	TmpAB0,Y
		sta	(TmpZP0,X)
		sta	(TmpZP0+4),Y

		lda	#$CC
		eor	#$CC
M291
		bne	M291		; error, ->
M292
		bmi	M292		; error, ->
		
		lda	#$F1
		sta	Test

		lda	#$CC
		eor	TmpZP0
M301
		bne	M301		; error, ->
M302
		bmi	M302		; error, ->

		lda	#$F2
		sta	Test

		lda	#$CC
		eor	TmpAB0
M311
		bne	M311		; error, ->
M312
		bmi	M312		; error, ->

		lda	#$F3
		sta	Test

		lda	#$CC
		eor	TmpZP0,X
M321
		bne	M321		; error, ->
M322
		bmi	M322		; error, ->

		lda	#$F4
		sta	Test

		lda	#$CC
		eor	TmpAB0,X
M331
		bne	M331		; error, ->
M332
		bmi	M332		; error, ->

		lda	#$F5
		sta	Test

		lda	#$CC
		eor	TmpAB0,Y
M341
		bne	M341		; error, ->
M342
		bmi	M342		; error, ->

		lda	#$F6
		sta	Test

		lda	#$CC
		eor	(TmpZP0,X)
M351
		bne	M351		; error, ->
M352
		bmi	M352		; error, ->

		lda	#$F7
		sta	Test

		lda	#$CC
		eor	(TmpZP0+4),Y
M361
		bne	M361		; error, ->
M362
		bmi	M362		; error, ->


;**  JMP ($xxxx)
		lda	#$F8
		sta	Test

		lda	#<(L000)
		sta	TmpAB0

		lda	#>(L000)
		sta	TmpAB0+1

		jmp	(TmpAB0)


;* Should not be executed at all
		nop
		sec
M371
		bcs	M371
		
		nop
M372
		nop

		
;**  JSR
		lda	#$F9
		sta	Test

		ldx	#$F8
		txs
		
		sec
		lda	#$FF
		jsr	L000a
M373
		bcs	M373
M374
		bne	M374

		nop


;**  PHP, PLP
		lda	#$FA
		sta	Test
		
		lda	#$FF
		pha
		pla
		sta	$06
		cmp	#$FF
XX01
		bne	XX01		    

		ldx	#0
		
		lda	#$FF
		pha		
		plp
		
		stx	$01F8
		
		php
		pla
		sta	$08
		cmp	#$FF
M381
		bne	M381
		
		tax

		lda	#0
		pha		
		plp
		
		stx	$01F8		; erase original value on purpose
		
		php
		pla
 and #$20		   	; avoid T65 bug
		cmp	#$20
M382
		bne	M382


;**  BRK, RTI
		lda	#$FB
		sta	Test
			    
		brk
		nop
		

;**  ADC  (3)             	Test ADC in decimal mode
		lda	#$FC
		sta	Test
			    
		sed
		
		clc
		lda	#$12
		adc	#$34
M390
		bcs	M390

		cmp	#$46
M391
		bne	M391
M392
		bcc	M392
M393
		bmi	M393
		
		sec
		adc	#$34
M395
		bcs	M395
M396
		bpl	M396

		sta	$04		; to check possible bug in T65 	
		cmp	#$81
M394
		bne	M394

		clc
		adc	#$19
		sta	$04		; to check possible bug in T65 	
M397
		bne	M397
M398
		bcc	M398
M399
		bmi	M399


;**  SBC  (3)             	Test SBC in decimal mode
		lda	#$FD
		sta	Test
			    




		lda	#$FE
		sta	Test
			    
L997
		jmp	L997


.ba $FFEC
;**  Part of testing JMP
L999
		jmp	L005
 
 
;* Should not be executed at all
		nop
		sec
L999c
		bcs	L999c
		
		nop



.ba $FFF4
IRQ
NMI
		nop
		
		lda	#$FF
		pha
		plp			; change flag register on purpose
		
		rti


.wo NMI
.wo Reset
.wo IRQ

.en
