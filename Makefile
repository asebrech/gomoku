.PHONY: dev release install zip unzip check-assets clean

check-assets:
	@if [ ! -f assets/backgrounds/ingame-background/*.png ] && [ -f assets/backgrounds/ingame-background/ingame-background.zip ]; then \
		$(MAKE) unzip; \
	elif [ ! -f assets/backgrounds/dolphin/*.png ] && [ -f assets/backgrounds/dolphin/dolphin.zip ]; then \
		$(MAKE) unzip; \
	fi

dev: check-assets
	cargo run

release: check-assets
	cargo build --release

install:
	@echo "Install target not yet implemented"

zip:
	cd assets/backgrounds/ingame-background && zip -r ingame-background.zip * -x "*.zip"
	cd assets/backgrounds/dolphin && zip -r dolphin.zip * -x "*.zip"
	@echo "Backgrounds compressed to zip files"

unzip:
	@if [ ! -f assets/backgrounds/ingame-background/*.png ]; then \
		cd assets/backgrounds/ingame-background && unzip -o ingame-background.zip; \
	fi
	@if [ ! -f assets/backgrounds/dolphin/*.png ]; then \
		cd assets/backgrounds/dolphin && unzip -o dolphin.zip; \
	fi
	@echo "Backgrounds extracted from zip files"

clean:
	find assets/backgrounds/ingame-background -type f ! -name "*.zip" -delete
	find assets/backgrounds/dolphin -type f ! -name "*.zip" -delete
	@echo "All images removed, zip files kept"
