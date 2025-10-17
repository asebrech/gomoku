#!/bin/bash

# Script to download and setup GStreamer binaries for Linux/macOS ARM64 and x86_64
# This creates a portable setup that doesn't require system-wide installation

set -e

GSTREAMER_VERSION="1.22.6"
DEPS_DIR="deps"
LIBS_DIR="$DEPS_DIR/lib"
INCLUDE_DIR="$DEPS_DIR/include"

# Detect architecture and OS
ARCH=$(uname -m)
OS=$(uname -s)

echo "Setting up GStreamer dependencies for Gomoku..."
echo "Detected: $OS $ARCH"

# Normalize architecture names
case "$ARCH" in
    x86_64|amd64)
        ARCH_NORMALIZED="x86_64"
        ;;
    aarch64|arm64)
        ARCH_NORMALIZED="arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Using normalized architecture: $ARCH_NORMALIZED"

# Create directories
mkdir -p "$LIBS_DIR"
mkdir -p "$INCLUDE_DIR"
mkdir -p "$DEPS_DIR/bin"

echo "Downloading GStreamer runtime libraries for $OS $ARCH_NORMALIZED..."

# Define library extensions based on OS
if [[ "$OS" == "Darwin" ]]; then
    LIB_EXT="dylib"
    LIB_PREFIX="lib"
    # macOS library paths
    SEARCH_PATHS=("/usr/local/lib" "/opt/homebrew/lib" "/usr/lib")
else
    LIB_EXT="so.0"
    LIB_PREFIX="lib"
    # Linux library paths based on architecture
    if [[ "$ARCH_NORMALIZED" == "arm64" ]]; then
        SEARCH_PATHS=("/usr/lib/aarch64-linux-gnu" "/usr/lib64" "/lib/aarch64-linux-gnu" "/lib64" "/usr/local/lib")
    else
        SEARCH_PATHS=("/usr/lib/x86_64-linux-gnu" "/usr/lib64" "/lib/x86_64-linux-gnu" "/lib64" "/usr/local/lib")
    fi
fi

# Core GStreamer libraries (without extension, we'll add it dynamically)
LIBS_BASE=(
    "gstreamer-1.0"
    "gstbase-1.0" 
    "gstapp-1.0"
    "gstvideo-1.0"
    "gstaudio-1.0"
    "gstpbutils-1.0"
    "gsttag-1.0"
    "gstriff-1.0"
    "glib-2.0"
    "gobject-2.0"
    "gio-2.0"
    "gmodule-2.0"
    "gthread-2.0"
    "orc-0.4"
)

# Wayland libraries (needed for compilation even if we use X11 at runtime)
WAYLAND_LIBS_BASE=(
    "wayland-client"
    "wayland-cursor"
    "wayland-egl"
)

# ALSA libraries (needed for audio)
ALSA_LIBS_BASE=(
    "asound"
)

# System libraries (udev for device detection)
SYSTEM_LIBS_BASE=(
    "udev"
)

# Optional X11 libraries (not Wayland since we're using X11 explicitly)
X11_LIBS_BASE=(
    "X11"
    "Xrandr"
    "Xcursor"
    "Xi"
    "Xinerama"
)

# Try to find system libraries first and copy them locally
echo "Looking for system GStreamer libraries..."
for lib_base in "${LIBS_BASE[@]}"; do
    lib_name="${LIB_PREFIX}${lib_base}.${LIB_EXT}"
    found=false
    
    # Try common library paths for the current architecture
    for path in "${SEARCH_PATHS[@]}"; do
        if [ -f "$path/$lib_name" ]; then
            echo "Found $lib_name at $path"
            cp "$path/$lib_name" "$LIBS_DIR/"
            found=true
            break
        fi
    done
    
    if [ "$found" = false ]; then
        echo "Warning: $lib_name not found in system paths"
    fi
done

# Also look for Wayland libraries (needed for compilation)
echo "Looking for Wayland libraries..."
for lib_base in "${WAYLAND_LIBS_BASE[@]}"; do
    lib_name="${LIB_PREFIX}${lib_base}.${LIB_EXT}"
    found=false
    
    for path in "${SEARCH_PATHS[@]}"; do
        if [ -f "$path/$lib_name" ]; then
            echo "Found $lib_name at $path"
            cp "$path/$lib_name" "$LIBS_DIR/"
            found=true
            break
        fi
    done
    
    if [ "$found" = false ]; then
        echo "Warning: $lib_name not found, will download"
    fi
done

# Look for ALSA libraries (needed for audio)
echo "Looking for ALSA libraries..."
for lib_base in "${ALSA_LIBS_BASE[@]}"; do
    lib_name="${LIB_PREFIX}${lib_base}.${LIB_EXT}"
    found=false
    
    for path in "${SEARCH_PATHS[@]}"; do
        if [ -f "$path/$lib_name" ]; then
            echo "Found $lib_name at $path"
            cp "$path/$lib_name" "$LIBS_DIR/"
            found=true
            break
        fi
    done
    
    if [ "$found" = false ]; then
        echo "Warning: $lib_name not found, will download"
    fi
done

# Look for system libraries (udev)
echo "Looking for system libraries..."
for lib_base in "${SYSTEM_LIBS_BASE[@]}"; do
    lib_name="${LIB_PREFIX}${lib_base}.${LIB_EXT}"
    found=false
    
    for path in "${SEARCH_PATHS[@]}"; do
        if [ -f "$path/$lib_name" ]; then
            echo "Found $lib_name at $path"
            cp "$path/$lib_name" "$LIBS_DIR/"
            found=true
            break
        fi
    done
    
    if [ "$found" = false ]; then
        echo "Warning: $lib_name not found, will download"
    fi
done

# Download function for different architectures and OS
download_packages() {
    if [[ "$OS" == "Darwin" ]]; then
        # macOS - use Homebrew bottles or build from source
        echo "For macOS, please install via Homebrew:"
        echo "brew install gstreamer gst-plugins-base gst-plugins-good pkg-config"
        return
    fi
    
    # Linux package downloads
    if [ ! -f "$LIBS_DIR/libgstreamer-1.0.$LIB_EXT" ]; then
        echo "System libraries not found. Attempting to download..."
        
        if [[ "$ARCH_NORMALIZED" == "arm64" ]]; then
            # ARM64 packages
            wget -q -O gstreamer-runtime.deb http://ports.ubuntu.com/pool/main/g/gstreamer1.0/libgstreamer1.0-0_1.20.3-0ubuntu1_arm64.deb
            wget -q -O gstreamer-plugins-base.deb http://ports.ubuntu.com/pool/main/g/gst-plugins-base1.0/libgstreamer-plugins-base1.0-0_1.20.3-0ubuntu1_arm64.deb
            wget -q -O glib2.deb http://ports.ubuntu.com/pool/main/g/glib2.0/libglib2.0-0_2.72.4-0ubuntu1_arm64.deb
            wget -q -O wayland.deb http://ports.ubuntu.com/pool/main/w/wayland/libwayland-client0_1.20.0-1ubuntu0.1_arm64.deb
        else
            # x86_64 packages
            wget -q -O gstreamer-runtime.deb http://archive.ubuntu.com/ubuntu/pool/main/g/gstreamer1.0/libgstreamer1.0-0_1.20.3-0ubuntu1_amd64.deb
            wget -q -O gstreamer-plugins-base.deb http://archive.ubuntu.com/ubuntu/pool/main/g/gst-plugins-base1.0/libgstreamer-plugins-base1.0-0_1.20.3-0ubuntu1_amd64.deb
            wget -q -O glib2.deb http://archive.ubuntu.com/ubuntu/pool/main/g/glib2.0/libglib2.0-0_2.72.4-0ubuntu1_amd64.deb
            wget -q -O wayland.deb http://archive.ubuntu.com/ubuntu/pool/main/w/wayland/libwayland-client0_1.20.0-1ubuntu0.1_amd64.deb
            wget -q -O alsa.deb http://archive.ubuntu.com/ubuntu/pool/main/a/alsa-lib/libasound2_1.2.6.1-1ubuntu1_amd64.deb
            wget -q -O udev.deb http://archive.ubuntu.com/ubuntu/pool/main/s/systemd/libudev1_249.11-0ubuntu3.12_amd64.deb
        fi
        
        # Extract libraries
        dpkg-deb -x gstreamer-runtime.deb temp_extract/
        dpkg-deb -x gstreamer-plugins-base.deb temp_extract/
        dpkg-deb -x glib2.deb temp_extract/
        dpkg-deb -x wayland.deb temp_extract/
        dpkg-deb -x alsa.deb temp_extract/
        dpkg-deb -x udev.deb temp_extract/
        
        # Copy libraries
        find temp_extract -name "*.so*" -type f | while read lib; do
            cp "$lib" "$LIBS_DIR/"
        done
        
        # Cleanup
        rm -rf temp_extract *.deb
    fi
}

# Call download function
download_packages

# Create pkg-config files
mkdir -p "$DEPS_DIR/lib/pkgconfig"

cat > "$DEPS_DIR/lib/pkgconfig/gstreamer-1.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GStreamer
Description: Streaming media framework
Version: 1.22.6
Requires: glib-2.0, gobject-2.0
Libs: -L\${libdir} -lgstreamer-1.0
Cflags: -I\${includedir}/gstreamer-1.0
EOF

cat > "$DEPS_DIR/lib/pkgconfig/gstreamer-app-1.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GStreamer App Library
Description: App library for GStreamer
Version: 1.22.6
Requires: gstreamer-1.0
Libs: -L\${libdir} -lgstapp-1.0
Cflags: -I\${includedir}/gstreamer-1.0
EOF

cat > "$DEPS_DIR/lib/pkgconfig/gstreamer-base-1.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GStreamer Base Library
Description: Base library for GStreamer
Version: 1.22.6
Requires: gstreamer-1.0
Libs: -L\${libdir} -lgstbase-1.0
Cflags: -I\${includedir}/gstreamer-1.0
EOF

cat > "$DEPS_DIR/lib/pkgconfig/gstreamer-video-1.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GStreamer Video Library
Description: Video library for GStreamer
Version: 1.22.6
Requires: gstreamer-1.0
Libs: -L\${libdir} -lgstvideo-1.0
Cflags: -I\${includedir}/gstreamer-1.0
EOF

cat > "$DEPS_DIR/lib/pkgconfig/glib-2.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GLib
Description: C Utility Library
Version: 2.72.4
Libs: -L\${libdir} -lglib-2.0
Cflags: -I\${includedir}/glib-2.0
EOF

cat > "$DEPS_DIR/lib/pkgconfig/gobject-2.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GObject
Description: GLib Type, Object, Parameter and Signal Library
Version: 2.72.4
Requires: glib-2.0
Libs: -L\${libdir} -lgobject-2.0
Cflags: -I\${includedir}/glib-2.0
EOF

cat > "$DEPS_DIR/lib/pkgconfig/gio-2.0.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GIO
Description: GLib Input, Output and Streaming Library
Version: 2.72.4
Requires: glib-2.0, gobject-2.0
Libs: -L\${libdir} -lgio-2.0
Cflags: -I\${includedir}/glib-2.0
EOF

# Create pkg-config files for Wayland (needed for compilation)
cat > "$DEPS_DIR/lib/pkgconfig/wayland-client.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: Wayland Client
Description: Wayland client side library
Version: 1.20.0
Libs: -L\${libdir} -lwayland-client
Cflags: -I\${includedir}
EOF

cat > "$DEPS_DIR/lib/pkgconfig/alsa.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: ALSA
Description: Advanced Linux Sound Architecture
Version: 1.2.6
Libs: -L\${libdir} -lasound
Cflags: -I\${includedir}/alsa
EOF

cat > "$DEPS_DIR/lib/pkgconfig/libudev.pc" << EOF
prefix=$PWD/$DEPS_DIR
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: libudev
Description: Library to access udev device information
Version: 249
Libs: -L\${libdir} -ludev
Cflags: -I\${includedir}
EOF

echo "Dependencies setup complete!"
echo "Run: export PKG_CONFIG_PATH=$PWD/$DEPS_DIR/lib/pkgconfig:\$PKG_CONFIG_PATH"
echo "Then: cargo build --release"
