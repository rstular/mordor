#!/usr/bin/python3

import argparse
import sqlite3

from argon2 import PasswordHasher


def generate_password(password: str) -> str:
    """Generate a hash of the password using Argon2"""
    ph = PasswordHasher()
    return ph.hash(password)


parser = argparse.ArgumentParser(description="Add a user to the database")
parser.add_argument("username", help="The username of the user to add")
parser.add_argument("password", help="The password of the user to add", nargs="?")
parser.add_argument(
    "-d",
    "--database-file",
    help="The database file to use",
    default="/var/web_server/mordor/mordor.db",
)
args = parser.parse_args()

password = generate_password(args.password if args.password else input("Password: "))

print(f"Adding user {args.username} to database {args.database_file}")

conn = sqlite3.connect(args.database_file)
sql = "INSERT INTO basic_login_user(username, password) VALUES(?,?)"
cur = conn.cursor()
cur.execute(sql, (args.username, password))
conn.commit()

print(f"User ID {cur.lastrowid} has been created")
