# M6-02: Optional Local PWA

**Status:** planned; optional  
**Dependencies:** M6-01

## Objective

Prove local browser installation and intentional offline-shell behavior without treating it as a desktop replacement.

## Scope

Add manifest, icons, supported-browser local install affordance, conservative shell caching, offline state, and recovery. Exclude offline run execution and data synchronization.

## Verification

- Install on a supported Chromium browser using local origin behavior.
- Launch standalone, stop local API, observe offline state, restart API, and recover.

## Acceptance criteria

- Installed shell has correct application metadata/icons.
- Offline mode never claims a server-backed run was created or completed.
- Browser-specific limitations are recorded in proof matrix.

## Evidence

_Pending execution._
