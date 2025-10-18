#!/bin/bash

# Cross-platform dependency setup for Gomoku
set -e

echo "Setting up GStreamer dependencies for Gomoku..."

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "Detected: $OS $ARCH"

# Create directory structure
DEPS_DIR="$(pwd)/deps"
LIBS_DIR="$DEPS_DIR/lib"
PKG_CONFIG_DIR="$LIBS_DIR/pkgconfig"

mkdir -p "$LIBS_DIR" "$PKG_CONFIG_DIR"

# macOS setup
if [[ "$OS" == "Darwin" ]]; then
    # Download GStreamer directly if not already present
    if [ ! -f "$LIBS_DIR/libgstreamer-1.0.dylib" ]; then
        echo "Downloading GStreamer runtime libraries..."
        
        # Create temp directory for download
        TEMP_DIR="$(mktemp -d)"
        cd "$TEMP_DIR"
        
        # Download GStreamer runtime package
        if command -v wget >/dev/null 2>&1; then
            wget -q "https://gstreamer.freedesktop.org/data/pkg/osx/1.24.8/gstreamer-1.0-1.24.8-universal.pkg" -O gstreamer.pkg
        elif command -v curl >/dev/null 2>&1; then
            curl -L -o gstreamer.pkg "https://gstreamer.freedesktop.org/data/pkg/osx/1.24.8/gstreamer-1.0-1.24.8-universal.pkg"
        else
            echo "❌ No download tool available (wget or curl needed)"
            exit 1
        fi
        
        # Extract the package
        if [ -f "gstreamer.pkg" ]; then
            echo "Extracting GStreamer libraries..."
            pkgutil --expand gstreamer.pkg extracted/
            
            # Extract all payload files
            find extracted/ -name "Payload" | while read payload; do
                echo "Extracting payload: $payload"
                cd "$(dirname "$payload")"
                cat Payload | gunzip -dc | cpio -i 2>/dev/null || true
                cd - >/dev/null
            done
            
            # Find and copy all dylib files
            find extracted/ -name "*.dylib" | while read lib; do
                cp "$lib" "$LIBS_DIR/"
                echo "Extracted: $(basename "$lib")"
            done
            
            # Also look for .so files and rename to .dylib if needed
            find extracted/ -name "*.so" | while read lib; do
                base=$(basename "$lib" .so)
                cp "$lib" "$LIBS_DIR/${base}.dylib"
                echo "Extracted and renamed: ${base}.dylib"
            done
            
            # Find and copy headers
            HEADER_DIR="$DEPS_DIR/include"
            mkdir -p "$HEADER_DIR"
            find extracted/ -name "gstreamer-1.0" -type d | while read headerdir; do
                if [ -d "$headerdir" ]; then
                    cp -r "$headerdir" "$HEADER_DIR/"
                    echo "Extracted headers to: $HEADER_DIR/gstreamer-1.0"
                fi
            done
            
            # Find and copy pkg-config files
            find extracted/ -name "*.pc" | while read pc; do
                cp "$pc" "$PKG_CONFIG_DIR/"
                echo "Extracted pkg-config: $(basename "$pc")"
            done
        fi
        
        # Cleanup
        cd - >/dev/null
        rm -rf "$TEMP_DIR"
        
        # Set prefix to our local installation
        GST_PREFIX="$DEPS_DIR"
    else
        echo "GStreamer libraries already present"
        GST_PREFIX="$DEPS_DIR"
    fi
    
    echo "Using GStreamer from: $GST_PREFIX"
    
    # Create pkg-config files for GStreamer
    cat > "$PKG_CONFIG_DIR/gstreamer-1.0.pc" << EOF
prefix=$GST_PREFIX
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GStreamer
Description: Streaming media framework
Version: 1.24.0
Requires: glib-2.0, gobject-2.0
Libs: -L\${libdir} -lgstreamer-1.0
Cflags: -I\${includedir}/gstreamer-1.0
EOF

    # Create component pkg-config files
    for component in base app video audio pbutils tag riff; do
        cat > "$PKG_CONFIG_DIR/gstreamer-${component}-1.0.pc" << EOF
prefix=$GST_PREFIX
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: GStreamer ${component}
Description: GStreamer ${component} library  
Version: 1.24.0
Requires: gstreamer-1.0
Libs: -L\${libdir} -lgst${component}-1.0
Cflags: -I\${includedir}/gstreamer-1.0
EOF
    done
    
    # Create GLib pkg-config files
    for component in glib-2.0 gobject-2.0 gio-2.0; do
        cat > "$PKG_CONFIG_DIR/${component}.pc" << EOF
prefix=$GST_PREFIX
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: ${component}
Description: GLib ${component} library
Version: 2.76.0
Libs: -L\${libdir} -l${component//-/_}
Cflags: -I\${includedir}/glib-2.0 -I\${libdir}/glib-2.0/include
EOF
    done

# Linux setup
elif [[ "$OS" == "Linux" ]]; then
    echo "Setting up GStreamer for Linux using system libraries..."
    
    # Check if system GStreamer is available
    GST_SYSTEM_PREFIX=""
    for prefix in /usr /usr/local; do
        if [ -f "$prefix/lib/x86_64-linux-gnu/libgstreamer-1.0.so" ] || [ -f "$prefix/lib/libgstreamer-1.0.so" ]; then
            GST_SYSTEM_PREFIX="$prefix"
            break
        fi
    done
    
    if [ -z "$GST_SYSTEM_PREFIX" ]; then
        echo "❌ GStreamer not found on system."
        echo "Please install GStreamer development packages:"
        echo "  Ubuntu/Debian: sudo apt install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev"
        echo "  Fedora: sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel"
        exit 1
    fi
    
    echo "Found GStreamer at: $GST_SYSTEM_PREFIX"
    
    # Find the actual library directory
    GST_LIB_DIR=""
    for libdir in "$GST_SYSTEM_PREFIX/lib/x86_64-linux-gnu" "$GST_SYSTEM_PREFIX/lib64" "$GST_SYSTEM_PREFIX/lib"; do
        if [ -f "$libdir/libgstreamer-1.0.so" ]; then
            GST_LIB_DIR="$libdir"
            break
        fi
    done
    
    # Find the include directory
    GST_INCLUDE_DIR=""
    for incdir in "$GST_SYSTEM_PREFIX/include"; do
        if [ -d "$incdir/gstreamer-1.0" ]; then
            GST_INCLUDE_DIR="$incdir"
            break
        fi
    done
    
    # Find pkg-config directory
    GST_PKGCONFIG_DIR=""
    for pcdir in "$GST_LIB_DIR/pkgconfig" "$GST_SYSTEM_PREFIX/lib/pkgconfig" "$GST_SYSTEM_PREFIX/lib64/pkgconfig" "$GST_SYSTEM_PREFIX/share/pkgconfig"; do
        if [ -f "$pcdir/gstreamer-1.0.pc" ]; then
            GST_PKGCONFIG_DIR="$pcdir"
            break
        fi
    done
    
    # Copy system libraries to local deps
    if [ -n "$GST_LIB_DIR" ]; then
        echo "Copying GStreamer libraries from $GST_LIB_DIR..."
        cp "$GST_LIB_DIR"/libgstreamer*.so* "$LIBS_DIR/" 2>/dev/null || true
        cp "$GST_LIB_DIR"/libgst*.so* "$LIBS_DIR/" 2>/dev/null || true
        cp "$GST_LIB_DIR"/libglib*.so* "$LIBS_DIR/" 2>/dev/null || true
        cp "$GST_LIB_DIR"/libgobject*.so* "$LIBS_DIR/" 2>/dev/null || true
        cp "$GST_LIB_DIR"/libgio*.so* "$LIBS_DIR/" 2>/dev/null || true
        
        # Copy GStreamer plugins
        echo "Copying GStreamer plugins..."
        GST_PLUGIN_DIRS=("$GST_LIB_DIR/gstreamer-1.0" "$GST_SYSTEM_PREFIX/lib/x86_64-linux-gnu/gstreamer-1.0" "$GST_SYSTEM_PREFIX/lib64/gstreamer-1.0")
        for plugin_dir in "${GST_PLUGIN_DIRS[@]}"; do
            if [ -d "$plugin_dir" ]; then
                mkdir -p "$LIBS_DIR/gstreamer-1.0"
                cp "$plugin_dir"/*.so "$LIBS_DIR/gstreamer-1.0/" 2>/dev/null || true
                echo "Copied plugins from: $plugin_dir"
                break
            fi
        done
    fi
    
    # Copy headers if available
    if [ -n "$GST_INCLUDE_DIR" ]; then
        HEADER_DIR="$DEPS_DIR/include"
        mkdir -p "$HEADER_DIR"
        if [ -d "$GST_INCLUDE_DIR/gstreamer-1.0" ]; then
            cp -r "$GST_INCLUDE_DIR/gstreamer-1.0" "$HEADER_DIR/" 2>/dev/null || true
        fi
        if [ -d "$GST_INCLUDE_DIR/glib-2.0" ]; then
            cp -r "$GST_INCLUDE_DIR/glib-2.0" "$HEADER_DIR/" 2>/dev/null || true
        fi
    fi
    
    # Copy or create pkg-config files
    if [ -n "$GST_PKGCONFIG_DIR" ]; then
        echo "Copying pkg-config files from $GST_PKGCONFIG_DIR..."
        cp "$GST_PKGCONFIG_DIR"/gstreamer*.pc "$PKG_CONFIG_DIR/" 2>/dev/null || true
        cp "$GST_PKGCONFIG_DIR"/glib*.pc "$PKG_CONFIG_DIR/" 2>/dev/null || true
        cp "$GST_PKGCONFIG_DIR"/gobject*.pc "$PKG_CONFIG_DIR/" 2>/dev/null || true
        cp "$GST_PKGCONFIG_DIR"/gio*.pc "$PKG_CONFIG_DIR/" 2>/dev/null || true
    fi
    
    # Set prefix to our local installation
    GST_PREFIX="$DEPS_DIR"
    
    # Update pkg-config files to point to our local installation
    for pc_file in "$PKG_CONFIG_DIR"/*.pc; do
        if [ -f "$pc_file" ]; then
            sed -i "s|^prefix=.*|prefix=$GST_PREFIX|g" "$pc_file"
            sed -i "s|^exec_prefix=.*|exec_prefix=\${prefix}|g" "$pc_file"
            sed -i "s|^libdir=.*|libdir=\${exec_prefix}/lib|g" "$pc_file"
            sed -i "s|^includedir=.*|includedir=\${prefix}/include|g" "$pc_file"
        fi
    done
    
    echo "Using GStreamer from: $GST_PREFIX"
else
    echo "❌ Unsupported OS: $OS"
    exit 1
fi

echo "Dependencies setup complete!"
echo "Run: export PKG_CONFIG_PATH=$PKG_CONFIG_DIR:\$PKG_CONFIG_PATH"
echo "Then: cargo build --release"
