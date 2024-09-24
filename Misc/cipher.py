def encrypt(plaintext):
    key = "2552"
    ciphertext = ""
    key_index = 0
    
    for char in plaintext:
        if char.isalpha():
            # Determine the shift based on the current key digit
            shift = int(key[key_index])
            
            # Shift the character
            if char.isupper():
                ciphertext += chr((ord(char) - 65 + shift) % 26 + 65)
            else:
                ciphertext += chr((ord(char) - 97 + shift) % 26 + 97)
            
            # Move to the next key digit
            key_index = (key_index + 1) % len(key)
        else:
            # Non-alphabetic characters remain unchanged
            ciphertext += char
    
    return ciphertext

def decrypt(ciphertext):
    key = "2552"
    plaintext = ""
    key_index = 0
    
    for char in ciphertext:
        if char.isalpha():
            # Determine the shift based on the current key digit
            shift = int(key[key_index])
            
            # Reverse the shift
            if char.isupper():
                plaintext += chr((ord(char) - 65 - shift) % 26 + 65)
            else:
                plaintext += chr((ord(char) - 97 - shift) % 26 + 97)
            
            # Move to the next key digit
            key_index = (key_index + 1) % len(key)
        else:
            # Non-alphabetic characters remain unchanged
            plaintext += char
    
    return plaintext

# Example usage
message = "NNAG"
encrypted = encrypt(message)
decrypted = decrypt(encrypted)

print(f"Original: {message}")
print(f"Encrypted: {encrypted}")
print(f"Decrypted: {decrypted}")