var module = module || { exports: {} };
module.exports = async function main() {
  console.log('call in host script')

  const ret = await Wapo.run(`
    var module = module || { exports: {} };
    module.exports = function main() {
      console.log('call in guest script');
      return 'hello';
    }
  `)
  console.log('back to host');
  console.log('the result of guest', ret)
}
