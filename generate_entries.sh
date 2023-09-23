#!/bin/bash

sea-orm-cli generate entity -o src/database/entity/generated --with-copy-enums --with-serde both
