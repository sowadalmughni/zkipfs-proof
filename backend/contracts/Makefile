# zkIPFS-Proof Smart Contracts Makefile

# Default target
.DEFAULT_GOAL := help

# Variables
NETWORK ?= localhost
PRIVATE_KEY ?= 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
RPC_URL ?= http://localhost:8545
ETHERSCAN_API_KEY ?= 
VERIFICATION_FEE ?= 0
MIN_SECURITY_LEVEL ?= 128

# Colors for output
RED := \033[31m
GREEN := \033[32m
YELLOW := \033[33m
BLUE := \033[34m
RESET := \033[0m

.PHONY: help install build test deploy verify clean format lint gas-report coverage docs

help: ## Show this help message
	@echo "$(BLUE)zkIPFS-Proof Smart Contracts$(RESET)"
	@echo "$(YELLOW)Available commands:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(RESET) %s\n", $$1, $$2}'

install: ## Install dependencies
	@echo "$(BLUE)Installing dependencies...$(RESET)"
	forge install foundry-rs/forge-std --no-commit
	forge install OpenZeppelin/openzeppelin-contracts --no-commit
	@echo "$(GREEN)Dependencies installed successfully!$(RESET)"

build: ## Build contracts
	@echo "$(BLUE)Building contracts...$(RESET)"
	forge build
	@echo "$(GREEN)Build completed!$(RESET)"

test: ## Run tests
	@echo "$(BLUE)Running tests...$(RESET)"
	forge test -vvv
	@echo "$(GREEN)Tests completed!$(RESET)"

test-gas: ## Run tests with gas reporting
	@echo "$(BLUE)Running tests with gas reporting...$(RESET)"
	forge test --gas-report
	@echo "$(GREEN)Gas report completed!$(RESET)"

test-coverage: ## Run test coverage
	@echo "$(BLUE)Running test coverage...$(RESET)"
	forge coverage
	@echo "$(GREEN)Coverage report completed!$(RESET)"

test-fuzz: ## Run fuzz tests
	@echo "$(BLUE)Running fuzz tests...$(RESET)"
	forge test --fuzz-runs 10000
	@echo "$(GREEN)Fuzz tests completed!$(RESET)"

deploy-local: ## Deploy to local network
	@echo "$(BLUE)Deploying to local network...$(RESET)"
	forge script script/Deploy.s.sol:Deploy --rpc-url $(RPC_URL) --private-key $(PRIVATE_KEY) --broadcast
	@echo "$(GREEN)Deployment to local network completed!$(RESET)"

deploy-testnet: ## Deploy to testnet
	@echo "$(BLUE)Deploying to $(NETWORK) testnet...$(RESET)"
	forge script script/Deploy.s.sol:Deploy --rpc-url $(RPC_URL) --private-key $(PRIVATE_KEY) --broadcast --verify --etherscan-api-key $(ETHERSCAN_API_KEY)
	@echo "$(GREEN)Deployment to $(NETWORK) completed!$(RESET)"

deploy-mainnet: ## Deploy to mainnet (requires confirmation)
	@echo "$(RED)WARNING: Deploying to mainnet!$(RESET)"
	@read -p "Are you sure you want to deploy to mainnet? (y/N): " confirm && [ "$$confirm" = "y" ]
	@echo "$(BLUE)Deploying to mainnet...$(RESET)"
	forge script script/Deploy.s.sol:Deploy --rpc-url $(RPC_URL) --private-key $(PRIVATE_KEY) --broadcast --verify --etherscan-api-key $(ETHERSCAN_API_KEY)
	@echo "$(GREEN)Deployment to mainnet completed!$(RESET)"

deploy-testing: ## Deploy with testing configuration
	@echo "$(BLUE)Deploying with testing configuration...$(RESET)"
	forge script script/Deploy.s.sol:Deploy --sig "deployForTesting()" --rpc-url $(RPC_URL) --private-key $(PRIVATE_KEY) --broadcast
	@echo "$(GREEN)Testing deployment completed!$(RESET)"

verify: ## Verify contracts on Etherscan
	@echo "$(BLUE)Verifying contracts...$(RESET)"
	forge verify-contract --chain-id 1 --num-of-optimizations 200 --watch --constructor-args $$(cast abi-encode "constructor(address,address)" $(OWNER_ADDRESS) $(FEE_RECIPIENT)) $(CONTRACT_ADDRESS) src/ZkIPFSVerifier.sol:ZkIPFSVerifier $(ETHERSCAN_API_KEY)
	@echo "$(GREEN)Verification completed!$(RESET)"

format: ## Format code
	@echo "$(BLUE)Formatting code...$(RESET)"
	forge fmt
	@echo "$(GREEN)Code formatted!$(RESET)"

lint: ## Lint code
	@echo "$(BLUE)Linting code...$(RESET)"
	forge fmt --check
	@echo "$(GREEN)Linting completed!$(RESET)"

gas-report: ## Generate gas report
	@echo "$(BLUE)Generating gas report...$(RESET)"
	forge test --gas-report > gas-report.txt
	@echo "$(GREEN)Gas report saved to gas-report.txt$(RESET)"

coverage: ## Generate coverage report
	@echo "$(BLUE)Generating coverage report...$(RESET)"
	forge coverage --report lcov
	@echo "$(GREEN)Coverage report generated!$(RESET)"

docs: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(RESET)"
	forge doc
	@echo "$(GREEN)Documentation generated in docs/$(RESET)"

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(RESET)"
	forge clean
	rm -rf cache out broadcast
	@echo "$(GREEN)Clean completed!$(RESET)"

anvil: ## Start local Anvil node
	@echo "$(BLUE)Starting Anvil local node...$(RESET)"
	anvil --host 0.0.0.0 --port 8545

# Network-specific deployments
deploy-sepolia: ## Deploy to Sepolia testnet
	$(MAKE) deploy-testnet NETWORK=sepolia RPC_URL=$$SEPOLIA_RPC_URL

deploy-goerli: ## Deploy to Goerli testnet
	$(MAKE) deploy-testnet NETWORK=goerli RPC_URL=$$GOERLI_RPC_URL

deploy-polygon: ## Deploy to Polygon mainnet
	$(MAKE) deploy-mainnet NETWORK=polygon RPC_URL=$$POLYGON_RPC_URL

deploy-arbitrum: ## Deploy to Arbitrum mainnet
	$(MAKE) deploy-mainnet NETWORK=arbitrum RPC_URL=$$ARBITRUM_RPC_URL

deploy-optimism: ## Deploy to Optimism mainnet
	$(MAKE) deploy-mainnet NETWORK=optimism RPC_URL=$$OPTIMISM_RPC_URL

deploy-base: ## Deploy to Base mainnet
	$(MAKE) deploy-mainnet NETWORK=base RPC_URL=$$BASE_RPC_URL

# Security and analysis
slither: ## Run Slither static analysis
	@echo "$(BLUE)Running Slither analysis...$(RESET)"
	slither .
	@echo "$(GREEN)Slither analysis completed!$(RESET)"

mythril: ## Run Mythril security analysis
	@echo "$(BLUE)Running Mythril analysis...$(RESET)"
	myth analyze src/ZkIPFSVerifier.sol
	@echo "$(GREEN)Mythril analysis completed!$(RESET)"

# Utility commands
flatten: ## Flatten contracts for verification
	@echo "$(BLUE)Flattening contracts...$(RESET)"
	forge flatten src/ZkIPFSVerifier.sol > flattened/ZkIPFSVerifier.sol
	@echo "$(GREEN)Contracts flattened to flattened/ directory$(RESET)"

size: ## Check contract sizes
	@echo "$(BLUE)Checking contract sizes...$(RESET)"
	forge build --sizes
	@echo "$(GREEN)Contract size check completed!$(RESET)"

# Environment setup
setup-env: ## Setup environment file
	@echo "$(BLUE)Setting up environment...$(RESET)"
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "$(YELLOW)Please edit .env file with your configuration$(RESET)"; \
	else \
		echo "$(YELLOW).env file already exists$(RESET)"; \
	fi

# Quick development workflow
dev: install build test ## Quick development setup

# CI/CD workflow
ci: build test lint coverage ## CI/CD pipeline

# Production deployment workflow
prod-deploy: build test lint gas-report deploy-mainnet verify ## Production deployment

# Show contract information
info: ## Show contract deployment information
	@echo "$(BLUE)Contract Information:$(RESET)"
	@if [ -f deployments/latest.json ]; then \
		cat deployments/latest.json | jq .; \
	else \
		echo "$(YELLOW)No deployment information found$(RESET)"; \
	fi

