var module = module || { exports: {} };
module.exports = async function main() {
  console.log('call in host script')
  const { value } = await Wapo.run(`
    var module = module || { exports: {} };
    module.exports = async function main() {
      console.log('call in guest script');
      await new Promise(resolve => setTimeout(resolve, 100));
      const ret = new Uint8Array([1, 2, 3, 4]);
      console.log('will return', ret);
      return ret;
    }
  `)
  console.log('back to host');
  console.log(value)
}
