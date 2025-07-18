## Fixes

- When authentication extensions are active, the Gateway would previously return 401 responses instead of 404 on unauthenticated requests to routes that do not exist. It now returns 404 responses. (https://github.com/grafbase/grafbase/pull/3296)
