# Production Release Checklist

## Required for every release candidate

- [ ] Clean git tree
- [ ] `Cargo.lock` committed
- [ ] `web/visualizer/package-lock.json` committed
- [ ] Production readiness gate passes
- [ ] Public test vectors pass
- [ ] SRS ceremony manifest check passes
- [ ] Deployment evidence pack exists
- [ ] Release-candidate evidence summary exists
- [ ] Audit packet exists
- [ ] Security checklist updated
- [ ] Changelog updated
- [ ] Release notes prepared

## Required for production-secure release

- [ ] External audit completed
- [ ] Critical findings resolved
- [ ] High findings resolved
- [ ] Side-channel review completed
- [ ] Production SRS artifact published
- [ ] Production SRS digest published
- [ ] Production ceremony transcript published
- [ ] Real long fuzz campaign logs archived
- [ ] Release artifacts checksummed
- [ ] Deployment evidence archived
- [ ] Final production approval recorded

## Current repository status

Current status:

    release-candidate capable

Not yet claimed:

    production-secure
    externally audited
    production SRS ceremony completed
