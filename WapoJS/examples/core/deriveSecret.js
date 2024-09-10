var module = module || { exports: {} };
module.exports = async function main() {
  console.log('call in host script')

  const r1 = Wapo.deriveSecret('hello')
  const r1Hex = Buffer.from(r1).toString('hex')
  console.log('r1', r1Hex)
}
