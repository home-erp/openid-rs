#!/bin/bash
set -e
rpmbuild -ba /root/rpmbuild/SPECS/openid-rs.spec && chown -R $1 /root/rpmbuild  