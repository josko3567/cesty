# Available platforms: WIN32, LINUX, OSX
PLATFORM = UNKNOWN
# Available architecures: AMD64, IA32, ARM
ARCH     = UNKNOWN
ifeq ($(OS),Windows_NT)
    PLATFORM = WIN32
    ifeq ($(PROCESSOR_ARCHITEW6432),AMD64)
        ARCH = AMD64
    else
        ifeq ($(PROCESSOR_ARCHITECTURE),AMD64)
            ARCH = AMD64
        endif
        ifeq ($(PROCESSOR_ARCHITECTURE),x86)
            ARCH = IA32
        endif
    endif
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        PLATFORM = LINUX
    endif
    ifeq ($(UNAME_S),Darwin)
        PLATFORM = OSX
    endif
    UNAME_P := $(shell uname -m)
    ifeq ($(UNAME_P),x86_64)
        ARCH = AMD64
    endif
    ifneq ($(filter %86,$(UNAME_P)),)
        ARCH = IA32
    endif
    ifneq ($(filter arm%,$(UNAME_P)),)
        ARCH = ARM
    endif
endif

PLATFORM_CHECK_IGNORE ?= F

ifeq ($(PLATFORM_CHECK_IGNORE),F)
	ifeq ($(PLATFORM),UNKNOWN)
$(error Unsupported platform. Run make with PLATFORM_CHECK_IGNORE=T \
to remove platform dependency.)
	endif
endif

ifeq ($(PLATFORM),WIN32)

	SHELL := powershell.exe
	.SHELLFLAGS := -Command

	BOLD   := 1
	BLACK  := Black
	RED    := Red
	GREEN  := Green
	YELLOW := Yellow
	BLUE   := Blue
	PURPLE := Magenta
	CYAN   := Cyan
	WHITE  := White

	PRINT  := Write-Host
	COLOR  = -ForegroundColor $(1) "$(2)"


else

	BOLD   := 0
	BLACK  := \e[$(BOLD);90m
	RED    := \e[$(BOLD);31m
	GREEN  := \e[$(BOLD);92m
	YELLOW := \e[$(BOLD);93m
	BLUE   := \e[$(BOLD);94m
	PURPLE := \e[$(BOLD);35m
	CYAN   := \e[$(BOLD);96m
	WHITE  := \e[$(BOLD);97m

	PRINT  := echo
	COLOR  = -e "$(1)$(2)\e[0;97m"

endif


#######################################
# START: Main settings.

DEBUG ?= T
GIT ?= T

### Available values: STATIC, DYNAMIC
LIBTYPE ?= STATIC
###

LIBRARY = lib
ifeq ($(PLATFORM),WIN32)
	ifeq ($(LIBTYPE),STATIC)
		LIBRARY := $(addsuffix .a, $(LIBRARY))
	else ifeq ($(LIBTYPE),DYNAMIC)
# TODO:
$(error Library type unknown, set to "$(LIBTYPE)". Must be either \
STATIC or DYNAMIC)
# LIBRARY := $(addsuffix .dll, $(LIBRARY))
	else
$(error Library type unknown, set to "$(LIBTYPE)". Must be either \
STATIC or DYNAMIC)
	endif
else
	ifeq ($(LIBTYPE),STATIC)
		LIBRARY := $(addsuffix .a, $(LIBRARY))
	else ifeq ($(LIBTYPE),DYNAMIC)
# TODO:
$(error Library type unknown, set to "$(LIBTYPE)". Must be either \
STATIC or DYNAMIC
# LIBRARY := $(addsuffix .so, $(LIBRARY))
	else
$(error Library type unspecified, set to $(LIBTYPE), must be either \
STATIC or DYNAMIC)
	endif
endif

BIN    ?= program
BINSRC := $(addsuffix .c, $(BIN))
ifeq ($(PLATFORM),WIN32)
	BINEXEC := $(addsuffix .exe, $(BIN))
else
	BINEXEC := $(addsuffix .out, $(BIN))
endif

CC  = clang
STD = gnu2x

CFLAGS = -fblocks -std=$(STD) -Wall -Wextra -Wpedantic \
	 -Wformat=2 -Wno-unused-parameter -Wshadow \
	 -Wwrite-strings -Wstrict-prototypes -Wold-style-definition \
	 -Wredundant-decls -Wnested-externs -Wmissing-include-dirs \
	 -pipe -Wno-unused-command-line-argument
ifeq ($(DEBUG), T) 
	CFLAGS += -g -DAG_DEV
else
	CFLAGS += -O3
endif

# External libraries used in linking:
LIBEXT = 
# ifeq ($(PLATFORM),WIN32)
# LIBEXT += ./ext/PDCurses/wincon/pdcurses.a
# endif

LIBEXTDIR = $(foreach file,$(LIBEXT), $(dir $(file)))

# Link:
LDFLAGS = -lm -lBlocksRuntime $(LIBEXT)
ifeq ($(PLATFORM),WIN32)
	LDFLAGS += ./ext/PDCurses/wincon/pdcurses.a
else
	LDFLAGS += -lncursesw
endif

# Source
SRCDIR = ./src

# END: Main settings.
#######################################

SRC = $(foreach D,$(SRCDIR),$(wildcard $(D)/*.c))   
OBJ = $(patsubst %.c,%.o,$(SRC))   

ifeq ($(PLATFORM),WIN32)
    REMOVE_ALL := Get-ChildItem * \
	-Include *.o, *.dll, *.exe, *.a, *.out, *.so -Recurse | Remove-Item
	COMMA := ,
	REMOVE := rm -Force \
	$(word 1,$(OBJ) $(LIBRARY) $(BIN)) \
	$(foreach f,$(wordlist 2,99999,$(OBJ) $(LIBRARY) $(BIN)),$(COMMA)$(f)) \
	-ErrorAction Ignore
else
	REMOVE_ALL := find . -name "*.o" -type f -delete; \
	find . -name "*.dll" -type f -delete; \
	find . -name "*.exe" -type f -delete; \
	find . -name "*.out" -type f -delete; \
	find . -name  "*.so" -type f -delete; \
	find . -name   "*.a" -type f -delete;
   	REMOVE := rm -f $(OBJ) $(LIBRARY) $(BINEXEC)
endif

# Link library...
$(LIBRARY): $(LIBEXT) $(OBJ)
	@$(PRINT) $(call COLOR,$(YELLOW),## Build info:)
	@$(PRINT) $(call COLOR,$(YELLOW),##     Platform ~ Arch: \
	$(PLATFORM) ~ $(ARCH))
	@$(PRINT) $(call COLOR,$(YELLOW),##     Library name ~ type: \
	$(LIBRARY) ~ $(LIBTYPE))
	@$(PRINT) $(call COLOR,$(YELLOW),##     Test file: $(BINSRC) ~ $(BINEXEC))
	@$(PRINT) $(call COLOR,$(YELLOW),##     Debug mode: $(DEBUG))
	@$(PRINT) $(call COLOR,$(YELLOW),##     Ignore platform check: \
	$(PLATFORM_CHECK_IGNORE))
	@$(PRINT) $(call COLOR,$(PURPLE),>> Linking library:)
	ar rcs $@ $(OBJ)

# Compile objects...
%.o: %.c
	@$(PRINT) $(call COLOR,$(GREEN),>> Compiling:) $<
	@$(CC) $(CFLAGS) -c $< -o $@ $(LDFLAGS) 

# Compile external dependencies...
ifeq ($(GIT),T)
$(LIBEXT): 
	@$(PRINT) $(call COLOR,$(PURPLE),>> git pull-ing external dependencies:)
	$(foreach dir,$(LIBEXTDIR),git -C ./$(dir) pull;)
	@$(PRINT) $(call COLOR,$(PURPLE),>> Compiling external dependencies:)
	$(foreach dir,$(LIBEXTDIR),make -C ./$(dir) WIDE=Y;)
else
$(LIBEXT): 
	@$(PRINT) $(call COLOR,$(YELLOW),#! skipping git pull-ing externel dep.)
	@$(PRINT) $(call COLOR,$(PURPLE),>> Compiling external dependencies:)
	$(foreach dir,$(LIBEXTDIR),make -C ./$(dir) WIDE=Y;)
endif

# Cleanup...
.PHONY: clean
clean: 
	@$(PRINT) $(call COLOR,$(RED),>> \
	Deleting local objects libraries & executables)
	$(REMOVE)

.PHONY: cleanall
cleanall:
	@$(PRINT) $(call COLOR,$(RED),>> \
	Deleting all objects libraries & executables)
	$(REMOVE_ALL)

# Create library and run test...
.PHONY: test
test: $(LIBRARY)
	@$(PRINT) $(call COLOR,$(PURPLE),>> Compiling/running $(BINSRC))
	$(CC) $(CFLAGS) $(BINSRC) -o $(BINEXEC) $(LIBRARY) $(LDFLAGS)
	@$(BINEXEC)

# Clean then test...
.PHONY: rebuild
rebuild: clean test