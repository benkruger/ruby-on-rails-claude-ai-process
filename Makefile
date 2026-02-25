VENV_PYTHON := .venv/bin/python3
PYTHON      := $(shell [ -x $(VENV_PYTHON) ] && echo $(VENV_PYTHON) || echo python3)

.PHONY: bump

bump:
ifndef NEW
	$(error Usage: make bump NEW=0.9.0)
endif
	$(PYTHON) hooks/bump-version.py $(NEW)
