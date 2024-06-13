import { createVerifiedFetch } from '@helia/verified-fetch'

async function main() {
    const verifiedFetch = await createVerifiedFetch({
        gateways: ['https://trustless-gateway.link', 'https://cloudflare-ipfs.com'],
    })
    const resp = await verifiedFetch('ipfs://baguqeeradnk3742vd3jxhurh22rgmlpcbzxvsy3vc5bzakwiktdeplwer6pa');
    console.log(await resp.text());
}

main().catch(console.error).finally(() => process.exit());
