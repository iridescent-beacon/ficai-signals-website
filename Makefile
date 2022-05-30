ODIR=./static
FDIR=./frontend

default: templates/fragments/cacheBuster.html

templates/fragments/cacheBuster.html: ${ODIR}/main.js ${ODIR}/main.css
	date '+%s' | tr -d '\n' > $@

# compile less into css
${ODIR}/%.css: ${FDIR}/%.less
	lessc $< $@

# create total javascript source
${ODIR}/main.js: ${FDIR}/*.ts
	tsc --strict --target es6 --out $@ $^

clean:
	rm -f ${ODIR}/main.js ${ODIR}/main.css

.PHONY: clean
