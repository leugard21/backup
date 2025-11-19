CC      := gcc
CFLAGS  := -Wall -Wextra -pedantic -std=c11 -O2
LDFLAGS := 
TARGET  := backup

SRC_DIR := src
SRC     := $(SRC_DIR)/main.c \
           $(SRC_DIR)/backup.c \
           $(SRC_DIR)/fs.c \
           $(SRC_DIR)/util.c

OBJ     := $(SRC:.c=.o)

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) $(LDFLAGS)

$(SRC_DIR)/%.o: $(SRC_DIR)/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJ) $(TARGET)
