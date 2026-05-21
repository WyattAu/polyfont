# polyfont-fonts

Cross-platform font discovery and management for polyfont.

Provides a unified `FontDiscovery` trait with platform-specific implementations
using system tools (`fc-list`, `system_profiler`, PowerShell) rather than
native library bindings.
