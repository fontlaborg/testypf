#!/bin/bash

# TestYPF Build Script for macOS
# Builds testypf GUI application with proper dependency handling

set -euo pipefail

# Change to script directory
cd "$(dirname "${BASH_SOURCE[0]}")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

command_exists() {
	command -v "$1" >/dev/null 2>&1
}

# Function to print colored output
print_status() {
	echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
	echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
	echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
	echo -e "${BLUE}$1${NC}"
}

require_command() {
	if ! command -v "$1" >/dev/null 2>&1; then
		print_error "Required command '$1' not found."
		return 1
	 fi
}

require_directory() {
	local dir=$1
	local hint=$2
	if [ ! -d "$dir" ]; then
		print_error "Missing directory: $dir"
		if [ -n "$hint" ]; then
			print_status "Hint: $hint"
		fi
		return 1
	fi
}

check_python_version() {
	local minimum=$1
	python3 - <<PY
import sys
req = tuple(map(int, "$minimum".split(".")))
if sys.version_info < req:
    sys.exit(1)
PY
	if [ $? -ne 0 ]; then
		print_error "Python $minimum+ is required (found $(python3 -V 2>&1))."
		return 1
	fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
	print_error "Cargo.toml not found. Please run this script from the testypf root directory."
	exit 1
fi

# Detect platform
PLATFORM="unknown"
if [[ "$OSTYPE" == "darwin"* ]]; then
	PLATFORM="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
	PLATFORM="linux"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
	PLATFORM="windows"
fi

print_status "Detected platform: $PLATFORM"

if [ "$PLATFORM" != "macos" ]; then
	print_error "This build script is optimized for macOS. Use the generic build script for other platforms."
	exit 1
fi

# Check prerequisites
print_status "Checking prerequisites..."

require_command cargo || exit 1
require_command python3 || exit 1
check_python_version "3.12" || exit 1

if ! command -v uv >/dev/null 2>&1; then
	print_warning "uv not found. Installing uv for Python package management..."
	curl -LsSf https://astral.sh/uv/install.sh | sh
	export PATH="$HOME/.cargo/bin:$PATH"
fi

# Check Xcode command line tools
if ! command_exists xcodebuild; then
	print_warning "Xcode command line tools may not be installed."
	print_status "Run 'xcode-select --install' if build fails."
fi

# Build function
build_component() {
	local component=$1
	local features=$2

	print_status "Building $component..."

	if [ -n "$features" ]; then
		cargo build -p "$component" --features "$features" "$@"
	else
		cargo build -p "$component" "$@"
	fi

	if [ $? -eq 0 ]; then
		print_status "$component built successfully"
	else
		print_error "Failed to build $component"
		exit 1
	fi
}

# Main build logic
print_header "
╔═══════════════════════════════════════╗
║         TestYPF macOS Build          ║
║     Typf GUI Testing Application     ║
╚═══════════════════════════════════════╝"

# Parse command line arguments
BUILD_TYPE="debug"
RUN_TESTS="no"
BUILD_RELEASE="no"
BUILD_GUI="no"
BUILD_CORE="no"
BUILD_ALL="yes"
SKIP_DEPS="no"
RUN_VERIFY="no"
RUN_DIAGNOSE="no"
RUN_FRESH_CHECK="no"
TYPF_FEATURES_DEFAULT="shaping-hb,render-opixa"
VERIFY_FONT_CANDIDATES=(
    "${TYPF_VERIFY_FONT:-}"
    "/System/Library/Fonts/Supplemental/Arial.ttf"
    "/System/Library/Fonts/Arial.ttf"
    "/Library/Fonts/Arial.ttf"
    "/System/Library/Fonts/Supplemental/Helvetica.ttc"
    "/System/Library/Fonts/Helvetica.ttc"
)

while [[ $# -gt 0 ]]; do
	case $1 in
	--release)
		BUILD_RELEASE="yes"
		BUILD_TYPE="release"
		shift
		;;
	--test)
		RUN_TESTS="yes"
		shift
		;;
	--gui)
		BUILD_GUI="yes"
		BUILD_ALL="no"
		shift
		;;
	--core)
		BUILD_CORE="yes"
		BUILD_ALL="no"
		shift
		;;
	--skip-deps)
		SKIP_DEPS="yes"
		shift
		;;
		--verify)
		RUN_VERIFY="yes"
		shift
		;;
	--diagnose)
		RUN_DIAGNOSE="yes"
		shift
		;;
	--fresh-check)
		RUN_FRESH_CHECK="yes"
		shift
		;;
	-h | --help)
		echo "TestYPF macOS Build Script"
		echo ""
		echo "Usage: $0 [OPTIONS]"
		echo ""
		echo "Options:"
		echo "  --release     Build in release mode"
		echo "  --test         Run tests after build"
		echo "  --gui          Build only GUI application"
		echo "  --core         Build only core library"
		echo "  --skip-deps    Skip dependency building"
		echo "  --verify       Verify typf/fontlift integration after build (uses TYPF_VERIFY_FONT override when set)"
		echo "  --diagnose     Run preflight checks only, no build"
		echo "  --fresh-check  Simulate fresh macOS setup: verify font dirs, temp uv venv, and required repos (no build)"
		echo "  --help         Show this help message"
		echo ""
		exit 0
		;;
	*)
		print_error "Unknown option: $1"
		exit 1
		;;
	esac
done

# Set build flags
BUILD_FLAGS=""
if [ "$BUILD_RELEASE" = "yes" ]; then
	BUILD_FLAGS="--release"
fi

if [ "$RUN_DIAGNOSE" = "yes" ]; then
	print_header "Running preflight diagnostics (no build)..."
	require_directory "../fontlift" "git clone git@github.com:fontlaborg/fontlift.git ../fontlift" || exit 1
	require_directory "../typf" "git clone git@github.com:fontlaborg/typf.git ../typf" || exit 1
	print_status "All required directories present."
	print_status "Python: $(python3 -V 2>&1)"
	print_status "Rustc: $(rustc -V 2>/dev/null || echo 'not found')"
	exit 0
fi

if [ "$RUN_FRESH_CHECK" = "yes" ]; then
	print_header "Fresh macOS environment check (no build)"
	require_directory "../fontlift" "git clone git@github.com:fontlaborg/fontlift.git ../fontlift" || exit 1
	require_directory "../typf" "git clone git@github.com:fontlaborg/typf.git ../typf" || exit 1

	for dir in "$HOME/Library/Fonts" "/Library/Fonts"; do
		if [ -d "$dir" ]; then
			if [ -w "$dir" ]; then
				print_status "Font directory writable: $dir"
			else
				print_warning "Font directory present but not writable without elevated privileges: $dir"
			fi
		else
			print_warning "Font directory missing: $dir (create it before installing fonts)"
		fi
	done

	tmpdir="$(mktemp -d)"
	print_status "Creating disposable uv virtualenv in $tmpdir..."
	if UV_PYTHON=python3 uv venv "$tmpdir/venv" >/dev/null 2>&1; then
		print_status "uv venv OK; Python version: $("$tmpdir/venv/bin/python" -V 2>&1)"
	else
		print_error "uv could not create a virtualenv; ensure curl-installed uv is on PATH and Python 3.12+ is available."
		rm -rf "$tmpdir"
		exit 1
	fi
	rm -rf "$tmpdir"

	print_status "Fresh-check complete. Run ./build.sh --verify after dependencies are present."
	exit 0
fi

# Build dependencies if needed
if [ "$SKIP_DEPS" = "no" ]; then
	print_status "Building dependencies..."

	# Check and build fontlift
	require_directory "../fontlift" "git clone git@github.com:fontlaborg/fontlift.git ../fontlift" || exit 1

	print_status "Building fontlift..."
	pushd ../fontlift >/dev/null
	./build.sh --core-only $BUILD_FLAGS
	popd >/dev/null

	# Check and build typf Python bindings
	require_directory "../typf" "git clone git@github.com:fontlaborg/typf.git ../typf" || exit 1

	print_status "Building typf Python bindings..."
	pushd ../typf/bindings/python >/dev/null

	# Create Python environment if it doesn't exist (prefer .venv if present)
	TYPF_VENV_DIR="venv"
	if [ -d ".venv" ]; then
		TYPF_VENV_DIR=".venv"
	fi

	if [ ! -d "$TYPF_VENV_DIR" ]; then
		print_status "Creating Python virtual environment at $TYPF_VENV_DIR..."
		UV_PYTHON=python3 uv venv "$TYPF_VENV_DIR"
	fi

	if [ ! -d "$TYPF_VENV_DIR" ]; then
		print_error "Failed to create typf virtualenv at $TYPF_VENV_DIR"
		exit 1
	fi

	# Activate environment and install dependencies
	source "$TYPF_VENV_DIR/bin/activate"
	uv pip install maturin -q
	uv pip install -e . -q

	# Build the Rust extension
	TYPF_FEATURES="${TYPF_FEATURES:-$TYPF_FEATURES_DEFAULT}"
	print_status "Using typf features: ${TYPF_FEATURES}"
	if [ "$BUILD_RELEASE" = "yes" ]; then
		maturin develop --release --features "${TYPF_FEATURES}"
	else
		maturin develop --features "${TYPF_FEATURES}"
	fi

	popd >/dev/null
fi

# Build components
if [ "$BUILD_ALL" = "yes" ] || [ "$BUILD_CORE" = "yes" ]; then
	if [ -n "$BUILD_FLAGS" ]; then
		cargo build -p testypf-core $BUILD_FLAGS
	else
		cargo build -p testypf-core
	fi

	if [ $? -eq 0 ]; then
		print_status "testypf-core built successfully"
	else
		print_error "Failed to build testypf-core"
		exit 1
	fi
fi

if [ "$BUILD_ALL" = "yes" ] || [ "$BUILD_GUI" = "yes" ]; then
	if [ -n "$BUILD_FLAGS" ]; then
		cargo build -p testypf-gui --features "iced/default" $BUILD_FLAGS
	else
		cargo build -p testypf-gui --features "iced/default"
	fi

	if [ $? -eq 0 ]; then
		print_status "testypf-gui built successfully"
	else
		print_error "Failed to build testypf-gui"
		exit 1
	fi
fi

# Run tests if requested
if [ "$RUN_TESTS" = "yes" ]; then
	print_status "Running tests..."

	if [ "$BUILD_RELEASE" = "yes" ]; then
		cargo test --workspace --release
	else
		cargo test --workspace
	fi

	if [ $? -eq 0 ]; then
		print_status "All tests passed"
	else
		print_error "Some tests failed"
		exit 1
	fi
fi

if [ "$RUN_VERIFY" = "yes" ]; then
	print_status "Running integration verification..."

	select_verify_font() {
		for font_path in "${VERIFY_FONT_CANDIDATES[@]}"; do
			if [ -n "$font_path" ] && [ -f "$font_path" ]; then
				echo "$font_path"
				return 0
			fi
		done
		return 1
	}

	TYPF_VENV_DIR="venv"
	if [ -d "../typf/bindings/python/.venv/bin" ]; then
		TYPF_VENV_DIR=".venv"
	fi

	if [ -d "../typf/bindings/python/$TYPF_VENV_DIR/bin" ]; then
		print_status "Checking typfpy Python module import..."
		if ! "../typf/bindings/python/$TYPF_VENV_DIR/bin/python" - <<'PY'
import typfpy
print("typfpy import OK, version:", getattr(typfpy, "__version__", "unknown"))
PY
		then
			print_error "typf Python module failed to import from venv."
			exit 1
		fi

		VERIFY_FONT=$(select_verify_font)
		if [ -z "$VERIFY_FONT" ]; then
			print_warning "No test font located. Set TYPF_VERIFY_FONT to an installed font path (e.g. /System/Library/Fonts/Supplemental/Arial.ttf)."
		else
			print_status "Exercising typfpy render_text with $VERIFY_FONT"
			if ! VERIFY_FONT="$VERIFY_FONT" "../typf/bindings/python/$TYPF_VENV_DIR/bin/python" - <<'PY'
import os
import typfpy

font_path = os.environ["VERIFY_FONT"]
engine = typfpy.Typf("harfbuzz", "opixa")
result = engine.render_text(
    "typf verify",
    font_path,
    18.0,
    (0, 0, 0, 255),
    None,
    4,
)

if not isinstance(result, dict):
    raise SystemExit("Unexpected typfpy render result; expected dict")

width = result.get("width")
height = result.get("height")
data = result.get("data")

if not width or not height or not data:
    raise SystemExit("typfpy render returned empty output")

print(f"typfpy render OK: {width}x{height} bytes={len(data)}")
PY
			then
				print_error "typf render test failed (see output above)."
				exit 1
			fi
		fi
	else
		print_warning "typf venv not found; skipping Python import verification."
	fi

	if [ -d "../fontlift/target" ]; then
		print_status "fontlift build artifacts present in ../fontlift/target"
	else
		print_warning "fontlift build artifacts not found; rerun dependencies build."
	fi
fi

# Print build summary
print_status "Build completed successfully!"
print_status "Build type: $BUILD_TYPE"

BINARY_NAME="testypf"
BINARY_PATH="target/$BUILD_TYPE/$BINARY_NAME"

if [ -f "$BINARY_PATH" ]; then
	print_status "GUI binary location: $BINARY_PATH"
	print_status "Binary size: $(du -h "$BINARY_PATH" | cut -f1)"
fi

# macOS-specific notes
print_status ""
print_status "macOS-specific notes:"
print_status "- CoreGraphics backend is available"
print_status "- Metal rendering backend available if features enabled"
print_status "- Font installation uses macOS native font management"

print_status ""
print_header "🚀 Quick Start"
print_status "1. Run the GUI application:"
print_status "   $ $BINARY_PATH"
print_status ""
print_status "2. Add font files using 'Add Fonts...' button or drag & drop"
print_status "3. Configure text and rendering settings"
print_status "4. Click 'Render Previews' to test Typf integration"
print_status ""
print_status "5. Documentation:"
print_status "- README.md: General overview"
print_status "- USAGE.md: Detailed usage instructions"
print_status "- PLAN.md: Architecture and future plans"

# Integration testing suggestions
print_status ""
print_header "🧪 Integration Testing"
print_status "To test integration with dependencies:"
print_status ""
print_status "FontLift integration:"
print_status "1. Font installation should work via GUI buttons"
print_status "2. Check /Library/Fonts or ~/Library/Fonts for installed fonts"
print_status ""
print_status "Typf integration:"
print_status "1. Try different rendering backends (Orge, Json, CoreGraphics)"
print_status "2. Test with various font formats (.ttf, .otf, .woff)"
print_status "3. Verify text renders correctly with custom sample text"

print_status ""
print_status "✨ Happy font testing with TestYPF! ✨"
