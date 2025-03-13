#!/bin/sh
set -e

ORIGINAL_MIX_ENV=$MIX_ENV

RELEASE=${1:-linux_debug}

# Run tests
export MIX_ENV=test
echo "Running tests..."
TEST_OUTPUT=$(mix test)

if echo "$TEST_OUTPUT" | grep -q "0 failures"; then
  echo "Tests passed. Proceeding with compilation..."
else
  echo "Tests failed. Aborting compilation."
  export MIX_ENV="$ORIGINAL_MIX_ENV"
  exit 1
fi

# Compile the Mix application
echo "Compiling Mix application for release: $RELEASE"
export MIX_ENV=prod
mix release "$RELEASE" --overwrite --quiet
cp rel/start_elevator.sh releases/"$RELEASE"/start_elevator.sh
chmod +x releases/"$RELEASE"/start_elevator.sh

export MIX_ENV="$ORIGINAL_MIX_ENV"
