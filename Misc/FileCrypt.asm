TITLE Encryption Project

; Description: encrypts and decrypts a file

; import statement
INCLUDE Irvine32.inc

; Author: Hillary Arurang

; .data is a segment that holds variables
.data

; max number of bytes to read in
BUFFER_SIZE = 5000

; variable of size 5000 bytes
buffer BYTE BUFFER_SIZE DUP(?)

; uninitialized variable that will hold bytes read in
bytesRead DWORD ?

; name of file to read in
inputName BYTE 5000 DUP(?)

; name of file to write to
outputName BYTE 5000 DUP (?)

; random generated key
key BYTE 16 DUP(?)

; strings used to prompt the user
askInputName BYTE "What is the name of your file you'd like to encrypt?", 0
askOutputName BYTE "What do you want your encrypted file titled?", 0
askToDecrypt BYTE "Do you want to decrypt the encrypted file? (y/n)", 0
askOverwrite BYTE "Do you want to run the program again? (y/n)", 0
keyMessage BYTE "Your key: ", 0

; response to ask prompts will be stored here
response BYTE ?

; compare askOverwrite response with ascii 'y'
; 121 is ascii for 'y'
yes = 121

; file handles which will contain file name and additional information used
; to read, write, open and close files
inputFileHandle DWORD ?
outputFileHandle DWORD ?

; random length for key
randomLength DWORD ?

; used to access different indices
index DWORD ?

; segment for exe code
.code

main PROC

	call getInputFileName
	call getOutputFileName
	call read
	call generateKey
	call write
	call askDecrypt
	call askRunAgain

	; exit calls MS windows instruction which halts the program
	; exit is a macro command defined in Irvine32.inc, not a MASM keyword
	exit

main ENDP

; asks the user a question and stores response in eax
getResponse PROC

	; display string in edx on terminal
	call WriteString

	; new line
	call crlf

	; point to eax, where data read in will be stored
	; eax will hold a memory operand
	mov edx, eax

	; how many characters to read
	mov ecx, 20

	; read terminal input into buffer
	call ReadString

	; return to caller
	ret

; end of procedure
getResponse ENDP

; ask the user for the name of the input file
getInputFileName PROC

	; point at string to be read
	mov edx, OFFSET askInputName

	; place memory operand that will hold response in eax
	mov eax, OFFSET inputName

	; display question and store response in inputName
	call getResponse

	; return to caller
	ret

; end procedure
getInputFileName ENDP

; ask the user for name of the output file
getOutputfileName PROC

	; point at string to be read
	mov edx, OFFSET askOutputName

	; place memory operand that will hold response in eax
	mov eax, OFFSET outputName

	; display question and store response in buffer
	call getResponse

	; return to caller
	ret

; end procedure
getOutputFileName ENDP

; creates a file handle for input file
openFile PROC

		; point at name of input file
		mov edx, OFFSET inputName

		; eax will now contain file handle
		call OpenInputFile

		; assign file handle to inputFileHandle
		mov inputFileHandle, eax

		; return to caller
		ret

; end of procedure
openFile ENDP

; read contents of file into buffer
read PROC

	; call procedure that creates a file handle in eax
	call openFile

	; point to buffer, bytes will be copied to buffer
	mov edx, OFFSET buffer

	; place how many times to loop in ecx
	mov ecx, BUFFER_SIZE

	; read file handle located in eax
	; eax contains number of bytes read in
	call ReadFromFile

	; move bytes read in to bytesRead
	mov bytesRead, eax

	; move the inputFileHandle back to eax
	mov eax, inputFileHandle

	; close input file
	call CloseFile

	; return to caller
	ret

; end procedure
read ENDP

; opens file handle for output file
openOutputFile PROC

	; move name of output file to edx
	mov edx, OFFSET outputName

	; eax now contains file handle
	call CreateOutputFile

	; place file handle in outputFileHandle
	mov outputFileHandle, eax

	; return to caller
	ret

; end of procedure
openOutputFile ENDP

; encrypts contents of buffer by xor
encrypt PROC

	; move number of bytes in buffer to ecx
	mov ecx, bytesRead

	; clear esi
	mov esi, 0

	; clear eax
	mov eax, 0

	; clear index memory operand
	mov index, 0

	; mov the first byte of key into ah
	mov ah, key[esi]

	L1:
		; xor each character in buffer with byte of key placed in ah
		xor buffer[esi], ah

		; save esi on stack
		push esi

		; add one to index
		inc index

		; overwrite esi with index value
		mov esi, index

		; clear eax
		mov eax, 0

		; move the next byte of key into ah
		mov ah, key[esi]

		; compare esi with 16
		cmp esi, LENGTHOF key

		; if esi equals 16
		je L2
			; clear index
			L2: mov index, 0

		; restore initial value of esi
		pop esi

		; increment esi
		inc esi

		loop L1

	; return to caller
	ret

; end procedure
encrypt ENDP

; writes encrypted data to output file
write PROC

	; encrypt contents of buffer
	call encrypt

	; creates a file handle in eax
	call openOutputFile

	; point to contents to read
	mov edx, OFFSET buffer

	mov ecx, 0

	; move bytes to read to ecx
	mov ecx, bytesRead

	; writes to file handle
	call WriteToFile

	; move output file handle back to eax
	mov eax, outputFileHandle

	; close output file
	call CloseFile

	; return to caller
	ret

; end of procedure
write ENDP

; ask the user if they would like to run the program again
askRunAgain PROC

	; move question to ask in edx
	mov edx, OFFSET askOverwrite

	; move memory operand that will hold response to askOverwrite in eax
	mov eax, OFFSET response

	; displays questions and stores response in response
	call getResponse

	; compare response with yes
	cmp response, yes

	; if response equals yes, jump to L1
	je L1

	; if response does not equal 'y' return to caller
	ret

	; Goes back to main
	L1: call Main

	; return to caller
	ret

; end of procedure
askRunAgain ENDP

; ask the user if they would like to xor their file again
askDecrypt PROC

	; move question to ask in edx
	mov edx, OFFSET askToDecrypt

	; move memory operand that will hold response to askOverwrite in eax
	mov eax, OFFSET response

	; displays questions and stores response in response
	call getResponse

	; compare response with yes
	cmp response, yes

	; if response equals yes, jump to L1
	je L1

	; if response does not equal 'y' return to caller
	ret

	; overwrite output file with decrypted text
	L1: call write

	; return to caller
	ret

; end of procedure
askDecrypt ENDP

generateLength PROC

	; get seconds that have elapsed since midnight and store in eax
	call getMSeconds

	; clear edx
	mov edx, 0

	; put 16 in ecx
	mov ebx, LENGTHOF key

	; eax / ebx --> mod is auto stored in edx
	div ebx

	; place mod in length
	mov randomLength, edx

	; return to caller
	ret

generateLength ENDP

generateKey PROC

	call generateLength

	; clear esi
	mov esi, 0

	; clear ecx
	mov ecx, 0

	; move randomLength to ecx
	mov ecx, randomLength

	; create a new seed for Random32
	call Randomize

	; first loop will generate first cycle
	L1:
		; clear eax after each loop
		mov eax, 0

		; place a random value in eax
		call Random32

		; store LSB in key index
		mov key[esi], al

		inc esi

		loop L1

	; store 16 in eax
	mov eax, LENGTHOF key

	; store randomLength in eax
	sub eax, randomLength

	; the difference will be how many more bytes we need in key
	mov ecx, eax

	mov esi, randomLength

	; second loop will generate remaining bytes follwoing the pattern generated in first cycle
	L2:
			; clear edx
			mov edx, 0

			; clear ebx
			mov ebx, 0

			; clear eax
			mov eax, 0

			; store randomLength in ebx
			mov ebx, randomLength

			;store value of esi in eax
			mov eax, esi

			; eax / ebx --> mod is auto stored in edx
			div ebx

			; store remainder in count
			mov index, edx

			; push esi value to stack
			push esi

			; assign esi value remainder
			mov esi, index

			; get byte at index[remainder] and store in ah
			mov al, key[esi]

			; restore value of esi
			pop esi

			; move value of key[remainder] into key[esi]
			mov key[esi], al

			; clear index
			mov index, 0

			inc esi

			loop L2

			; print random key
			call printKey

	; return to caller
	ret

generateKey ENDP

printKey PROC

	; string to read
	mov edx, OFFSET keyMessage

	; display key on terminal
	call WriteString

	; write new line
	call crlf

	; number of bytes to read
	mov ecx, LENGTHOF key

	; clear esi
	mov esi, 0

	L1:
		; clear eax
		mov eax, 0

		; al now contains element to be read
		mov al, key[esi]

		; number of bytes to read in ebx
		mov ebx, TYPE key

		; display byte in terminal
		call WriteHexB

		; increment esi
		inc esi

		; new line
		call crlf

		loop L1

		; return to caller
		ret

printKey ENDP

; end of program
end main