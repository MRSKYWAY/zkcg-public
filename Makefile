FOUNDRY_BIN ?= $(HOME)/.foundry/bin

.PHONY: gpu-check onchain-test onchain-demo onchain-rwa-demo

gpu-check:
	bash ./scripts/gpu_smoke.sh

onchain-test:
	$(FOUNDRY_BIN)/forge test

onchain-demo:
	bash ./scripts/demo_private_loan_onchain.sh

onchain-rwa-demo:
	bash ./scripts/demo_rwa_transfer_onchain.sh
