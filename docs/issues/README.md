# Known Issues Index

**Zero-Latency Document Search System**  
**Last Updated**: August 30, 2025  

## 🚨 Critical Issues (Require Immediate Attention)

### Metadata & Collection Management
| Issue | Severity | Status | Document |
|-------|----------|--------|----------|
| Empty Search Metadata | High | 🔍 Open | [metadata-issues.md](metadata-issues.md) |
| Document ID Not Preserved | High | 🔍 Open | [metadata-issues.md](metadata-issues.md) |
| Collection Association Lost | High | 🔍 Open | [metadata-issues.md](metadata-issues.md) |

## ⚠️ Medium Priority Issues

### Search & Filtering
| Issue | Severity | Status | Document |
|-------|----------|--------|----------|
| CLI Collection Filtering | Medium | 📝 Documented | [search-issues.md](search-issues.md) |

## ✅ Resolved Issues

### Search Pipeline
| Issue | Severity | Status | Document |
|-------|----------|--------|----------|
| Search Limit Bug | Medium | ✅ Fixed | [../implementation/SEARCH_LIMIT_BUG_FIX.md](../implementation/SEARCH_LIMIT_BUG_FIX.md) |

## 📋 Issue Categories

### By Component
- **Metadata Management**: [metadata-issues.md](metadata-issues.md)
- **Search & Filtering**: [search-issues.md](search-issues.md)
- **Protocol Compliance**: [protocol-issues.md](protocol-issues.md)

### By Severity
- **Critical (High)**: Issues that break core functionality or data integrity
- **Medium**: Issues that affect user experience but don't break core features
- **Low**: Minor issues or improvement opportunities

## 🔄 Issue Lifecycle

1. **🔍 Open**: Issue identified and under investigation
2. **📋 Planned**: Fix planned and prioritized
3. **🚧 In Progress**: Actively being worked on
4. **✅ Fixed**: Issue resolved and verified
5. **📝 Documented**: Issue documented but not yet addressed

## 📊 Current Status Summary

- **Total Issues**: 4
- **Critical**: 3 (75%)
- **Medium**: 1 (25%)
- **Open**: 4 (100%)
- **Fixed**: 0 (0%)

## 🔗 Related Documentation

- [Architecture Overview](../CURRENT_ARCHITECTURE.md)
- [Implementation Details](../implementation/)
- [Architecture Decisions](../adr/)
