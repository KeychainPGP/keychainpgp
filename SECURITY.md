# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in KeychainPGP, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email: **security@keychainpgp.org**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- **Acknowledgment**: within 48 hours
- **Initial assessment**: within 7 days
- **Fix and disclosure**: within 90 days

## Scope

The following are in scope:
- Cryptographic weaknesses in key generation, encryption, or decryption
- Private key exfiltration or leakage
- Clipboard content leakage beyond documented threat model
- Memory safety issues (buffer overflows, use-after-free)
- Authentication or authorization bypasses
- Dependency vulnerabilities with exploitable impact

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest release | Yes |
| Previous minor | Security fixes only |
| Older | No |

## Recognition

We acknowledge security researchers who report valid vulnerabilities in our release notes and CHANGELOG (unless anonymity is requested).
