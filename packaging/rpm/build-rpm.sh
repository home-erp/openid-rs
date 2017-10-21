#!/bin/bash

user_id=$(id -u)

tar -cf openid-rs.tar ../../src ../../tests ../../Cargo.* ../../build.rs
mkdir -p ../../target/rpms 
docker build -t build-openid-rs .
docker run --rm -it -v $PWD/openid-rs.tar:/root/rpmbuild/SOURCES/openid-rs.tar -v $PWD/openid-rs.spec:/root/rpmbuild/SPECS/openid-rs.spec -v $PWD/out:/root/rpmbuild/RPMS build-openid-rs  bash /usr/local/bin/entrypoint.sh $user_id

rm -rf ../../target/rpms
mv out/* ../../target/rpms
rm -rf out
rm openid-rs.tar