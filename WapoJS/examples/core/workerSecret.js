var module = module || { exports: {} };
module.exports = async function main() {
  console.log('call in host script')

  console.log('Wapo.workerSecret', Wapo.workerSecret)
  Wapo.workerSecret = 'modified'
  console.log('Wapo.workerSecret', Wapo.workerSecret)

  const ret = await Wapo.run(`
    var module = module || { exports: {} };
    module.exports = async function main() {
      console.log('call in guest script');
      console.log('Wapo.workerSecret', Wapo.workerSecret)
      Wapo.workerSecret = 'modified'
      console.log('Wapo.workerSecret', Wapo.workerSecret)
    }
  `)
  console.log('back to host');
  console.log('the result of guest', ret)
}
