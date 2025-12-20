#!/bin/bash

# Fix catch clauses that don't specify exception type

# Find all files with catch clauses that need fixing
grep -rl '\} catch (e' lib test integration_test --include="*.dart" 2>/dev/null | while read -r file; do
  echo "Fixing $file..."
  # Replace } catch (e with } on Exception catch (e
  sed -i 's/} catch (e/} on Exception catch (e/g' "$file"
done

echo "Done fixing catch clauses!"
