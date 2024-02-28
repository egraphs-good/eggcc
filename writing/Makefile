BUILD_MAIN_TEX := pdflatex -file-line-error -interaction=nonstopmode main.tex

main.pdf: main.tex semantics.tex macros.tex bcprules.sty
	$(BUILD_MAIN_TEX)

force: .FORCE
	$(BUILD_MAIN_TEX)

.PHONY: force .FORCE
.FORCE:

