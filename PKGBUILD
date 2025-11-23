pkgname=backup
pkgver=1.0.0
pkgrel=1
pkgdesc="Multi-threaded system backup utility written in Rust"
arch=('x86_64')
url="https://github.com/leugard21/backup"
license=('MIT')
depends=()
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
  tar xf "$pkgname-$pkgver.tar.gz"
}

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 "target/release/backup" "$pkgdir/usr/bin/backup"
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
