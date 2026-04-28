#!/bin/bash

# Скрипт автоматической настройки безопасной среды (SSH + GPG) на macOS
# Автор: Gemini CLI Agent

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

echo -e "${BLUE}=== Secure Environment Setup for macOS ===${NC}\n"

# 1. Проверка зависимостей
log "Checking dependencies..."
command -v brew >/dev/null 2>&1 || error "Homebrew not found. Install it first: https://brew.sh"
command -v gh >/dev/null 2>&1 || brew install gh
command -v gpg >/dev/null 2>&1 || brew install gnupg
command -v pinentry-mac >/dev/null 2>&1 || brew install pinentry-mac

# 2. Проверка аутентификации GitHub и необходимых Scope
log "Checking GitHub CLI authentication and scopes..."
if ! gh auth status >/dev/null 2>&1; then
    warn "Not authenticated with GitHub CLI."
    gh auth login
fi

check_scopes() {
    local scopes=$(gh auth status -v 2>&1 | grep "Token scopes" | cut -d: -f2)
    local missing=()
    [[ $scopes != *"admin:public_key"* ]] && missing+=("admin:public_key")
    [[ $scopes != *"write:gpg_key"* ]] && missing+=("write:gpg_key")
    
    if [ ${#missing[@]} -ne 0 ]; then
        warn "Missing required scopes: ${missing[*]}"
        echo -e "\n${YELLOW}Please run the following command to grant permissions:${NC}"
        echo -e "${BLUE}gh auth refresh -s admin:public_key -s write:gpg_key${NC}\n"
        read -p "Press Enter after you have completed the authentication in your browser..."
        check_scopes
    fi
}
check_scopes
success "GitHub CLI is ready with correct scopes."

# 3. Настройка почты Git
GH_USER=$(gh api user -q .login)
GH_EMAIL=$(gh api user/emails -q '.[] | select(.primary==true) | .email')
[ -z "$GH_EMAIL" ] && GH_EMAIL=$(gh api user -q .email)

log "Using GitHub details: User: $GH_USER, Email: $GH_EMAIL"
git config --global user.name "$GH_USER"
git config --global user.email "$GH_EMAIL"

# 4. Настройка SSH
log "Setting up SSH (Ed25519)..."
SSH_KEY="$HOME/.ssh/id_ed25519"
if [ ! -f "$SSH_KEY" ]; then
    ssh-keygen -t ed25519 -C "$GH_EMAIL" -f "$SSH_KEY" -N ""
    success "Generated new SSH key."
else
    warn "SSH key already exists at $SSH_KEY"
fi

# Настройка SSH Config для macOS Keychain
log "Configuring SSH to use macOS Keychain..."
[ ! -d "$HOME/.ssh" ] && mkdir -p "$HOME/.ssh"
grep -q "UseKeychain yes" "$HOME/.ssh/config" 2>/dev/null || cat >> "$HOME/.ssh/config" <<EOF

Host github.com
  AddKeysToAgent yes
  UseKeychain yes
  IdentityFile $SSH_KEY
EOF

ssh-add --apple-use-keychain "$SSH_KEY" >/dev/null 2>&1

# Добавление SSH ключа в GitHub
if ! gh ssh-key list | grep -q "$(cat ${SSH_KEY}.pub | awk '{print $2}')"; then
    log "Adding SSH key to GitHub..."
    gh ssh-key add "${SSH_KEY}.pub" --title "macOS-$(hostname)-$(date +%F)"
    success "SSH key added to GitHub."
else
    success "SSH key already present in GitHub."
fi

# 5. Настройка GPG
log "Setting up GPG (Ed25519)..."

# Настройка gpg-agent для macOS Keychain
mkdir -p ~/.gnupg
chmod 700 ~/.gnupg
echo "pinentry-program $(which pinentry-mac)" > ~/.gnupg/gpg-agent.conf
killall gpg-agent >/dev/null 2>&1 || true

# Генерация ключа, если его нет
EXISTING_GPG=$(gpg --list-secret-keys --with-colons | grep '^fpr' | head -n 1 | cut -d: -f10)

if [ -z "$EXISTING_GPG" ]; then
    log "Generating new GPG Ed25519 key..."
    # Создаем временный файл конфигурации для batch генерации
    cat > /tmp/gpg_batch <<EOF
     %no-protection
     Key-Type: EdDSA
     Key-Curve: ed25519
     Key-Usage: sign
     Name-Real: $GH_USER
     Name-Email: $GH_EMAIL
     Expire-Date: 0
     %commit
EOF
    gpg --batch --generate-key /tmp/gpg_batch
    rm /tmp/gpg_batch
    GPG_KEY_ID=$(gpg --list-secret-keys --keyid-format=long | grep 'sec' | head -n 1 | awk '{print $2}' | cut -d/ -f2)
else
    GPG_KEY_ID=$(gpg --list-secret-keys --keyid-format=long | grep 'sec' | head -n 1 | awk '{print $2}' | cut -d/ -f2)
    warn "Using existing GPG key: $GPG_KEY_ID"
fi

# Добавление в GitHub
if ! gh gpg-key list | grep -q "$GPG_KEY_ID"; then
    log "Exporting and adding GPG key to GitHub..."
    gpg --armor --export "$GPG_KEY_ID" | gh gpg-key add -
    success "GPG key added to GitHub."
else
    success "GPG key already present in GitHub."
fi

# 6. Финальная настройка Git
log "Finalizing Git configuration..."
git config --global user.signingkey "$GPG_KEY_ID"
git config --global commit.gpgsign true
git config --global tag.gpgsign true

# 7. Вывод результатов
echo -e "\n${GREEN}=== SETUP COMPLETE ===${NC}"
echo -e "${BLUE}SSH Fingerprint:${NC} $(ssh-keygen -l -f ${SSH_KEY}.pub)"
echo -e "${BLUE}GPG Key ID:${NC}      $GPG_KEY_ID"
echo -e "${BLUE}Git Email:${NC}       $(git config --global user.email)"
echo -e "${BLUE}Signing Enabled:${NC} $(git config --global commit.gpgsign)"

echo -e "\n${YELLOW}Testing signing...${NC}"
TEST_DIR="/tmp/git-sign-test"
mkdir -p "$TEST_DIR" && cd "$TEST_DIR" && git init >/dev/null
touch test && git add test
if git commit -S -m "test" >/dev/null 2>&1; then
    success "GPG signing test passed!"
else
    error "GPG signing test failed. Check ~/.gnupg/gpg-agent.conf"
fi
rm -rf "$TEST_DIR"

echo -e "\n${GREEN}Everything is ready! All keys are managed by macOS Keychain.${NC}"
