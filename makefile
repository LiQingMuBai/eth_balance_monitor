# 增强版 Makefile for Rust 项目

# 项目配置
PROJECT := your_app
CARGO := cargo

# 自动检测操作系统
ifeq ($(OS),Windows_NT)
    PLATFORM := Windows
    BIN_EXT := .exe
    SCRIPT_EXT := .bat
    MKDIR := mkdir
    RMDIR := rmdir /s /q
    CP := copy
    RM := del /q
    SEP := \\
else
    PLATFORM := Linux
    BIN_EXT :=
    SCRIPT_EXT := .sh
    MKDIR := mkdir -p
    RMDIR := rm -rf
    CP := cp
    RM := rm -f
    SEP := /
endif

# 路径定义
TARGET_DEBUG := target$(SEP)debug$(SEP)$(PROJECT)$(BIN_EXT)
TARGET_RELEASE := target$(SEP)release$(SEP)$(PROJECT)$(BIN_EXT)
SCRIPTS_DIR := scripts$(SEP)$(PLATFORM)
RUN_SCRIPT := run$(SCRIPT_EXT)

# 默认目标
all: build

# 调试构建
debug:
	$(CARGO) build

# 发布构建
build:
	$(CARGO) build --release
	@echo "构建完成: $(TARGET_RELEASE)"

# 运行程序
run: build
	@echo "运行程序..."
ifeq ($(PLATFORM),Windows)
	$(TARGET_RELEASE)
else
	./$(TARGET_RELEASE)
endif

# 清理构建文件
clean:
	$(CARGO) clean
	@echo "已清理构建文件"

# 生成启动脚本
scripts:
	$(MKDIR) $(SCRIPTS_DIR)
ifeq ($(PLATFORM),Windows)
	@echo @echo off > $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
	@echo REM Windows 启动脚本 >> $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
	@echo $(TARGET_RELEASE) >> $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
	@echo pause >> $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
else
	@echo '#!/bin/bash' > $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
	@echo '# Linux 启动脚本' >> $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
	@echo './$(TARGET_RELEASE)' >> $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
	chmod +x $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)
endif
	@echo "已生成 $(PLATFORM) 启动脚本: $(SCRIPTS_DIR)$(SEP)$(RUN_SCRIPT)"

# 运行测试
test:
	$(CARGO) test

# 代码检查
check:
	$(CARGO) check

# 更新依赖
update:
	$(CARGO) update

# 安装工具链
install-toolchain:
	rustup update
	rustup component add rustfmt clippy

# 格式化代码
fmt:
	$(CARGO) fmt

# lint 检查
lint:
	$(CARGO) clippy

# 显示帮助信息
help:
	@echo "可用命令:"
	@echo "  make debug          - 调试构建"
	@echo "  make build         - 发布构建 (默认)"
	@echo "  make run           - 构建并运行程序"
	@echo "  make clean         - 清理构建文件"
	@echo "  make scripts       - 生成启动脚本"
	@echo "  make test          - 运行测试"
	@echo "  make check         - 检查代码"
	@echo "  make update        - 更新依赖"
	@echo "  make install-toolchain - 安装工具链"
	@echo "  make fmt           - 格式化代码"
	@echo "  make lint          - 运行 clippy 检查"
	@echo "  make help          - 显示此帮助信息"

.PHONY: all debug build run clean scripts test check update install-toolchain fmt lint help