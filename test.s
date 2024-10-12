mov v5, 30
start:
mov v4, 0

loop:
gca	v4
clr

draw v0, v1, 5
add v4, 1

sdt v5

waitloop:
gdt v6
be v6, 0
j waitloop


be v4, 16
j loop
j start
