var module = module || { exports: {} };
module.exports = async function main() {
  console.log('call in host script')

  const ret = await Wapo.run(`
    var module = module || { exports: {} };
    module.exports = async function main() {
      console.log('call in guest script');
      await new Promise(resolve => setTimeout(resolve, 100));
      return 42
    }
  `)
  console.log('back to host');
  console.log('the result of guest', ret)
}
