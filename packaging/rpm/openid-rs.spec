Name:           openid-rs
Version:        0.1.0
Release:        1%{?dist}
Summary:        Openid server written in rust  

License:        GPLv3
Source0:        openid-rs.tar

%description
Openid connect server written in rust 
%prep
cp ../SOURCES/openid-rs.tar ./
tar -xf openid-rs.tar
%build
cargo build --release
%install
mkdir -p %{buildroot}%{_bindir}/
cp -p target/release/openid-rs  %{buildroot}%{_bindir}/
%files
%{_bindir}/*
