#!/usr/bin/python3

import sys
import getpass

from argon2 import PasswordHasher


def generate_password(password: str) -> str:
    """Generate a hash of the password using Argon2"""
    ph = PasswordHasher()
    return ph.hash(password)


def main():
    if len(sys.argv) > 2:
        print("Usage: python generate_password.py [password]")
        sys.exit(1)

    password = sys.argv[1] if len(sys.argv) == 2 else getpass.getpass("Password: ")
    print(generate_password(password=password))


if __name__ == "__main__":
    main()
