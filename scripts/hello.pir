// === pIR Hello World & Features Test ===
// Tests: var declaration, print, strict prefix ops, while loop, function call

v msg "Hello from pIR (Pseudo Intermediate Representation)!"
p msg

v x 10
v y (+ x 5)
p y

? (> y 12) [
    p "y is greater than 12"
] [
    p "y is 12 or less"
]

v i 3
w (> i 0) [
    p i
    = i (- i 1)
]

f greet (name) [
    r (+ "Hello, " name)
]

v greeting (greet "pIR Engineer")
p greeting
