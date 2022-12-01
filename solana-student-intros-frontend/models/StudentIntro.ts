import * as borsh from '@project-serum/borsh'
import { PublicKey } from "@solana/web3.js"
import { STUDENT_INTRO_PROGRAM_ID } from "../utils/constants"

export class StudentIntro {
    name: string;
    message: string;
    reviewer: PublicKey;

    constructor(name: string, message: string, reviewer: PublicKey) {
        this.name = name;
        this.message = message;
        this.reviewer = reviewer;
    }

    async publicKey(): Promise<PublicKey> {
        console.log("reviewer:", this.reviewer.toBase58())
        console.log("name:", this.name)
        return (
          await PublicKey.findProgramAddress(
            [this.reviewer.toBuffer(), Buffer.from(this.name)],
            new PublicKey(STUDENT_INTRO_PROGRAM_ID)
          )
        )[0]
      }

    static mocks: StudentIntro[] = [
        new StudentIntro('Elizabeth Holmes', `Learning Solana so I can use it to build sick NFT projects.`,new PublicKey("EurMFhvwKScjv469XQoUm1Qj6PFJQoVwXYmdgeXCqg5m")),
        new StudentIntro('Jack Nicholson', `I want to overhaul the world's financial system. Lower friction payments/transfer, lower fees, faster payouts, better collateralization for loans, etc.`,new PublicKey("EurMFhvwKScjv469XQoUm1Qj6PFJQoVwXYmdgeXCqg5m")),
        new StudentIntro('Terminator', `i'm basically here to protect`,new PublicKey("EurMFhvwKScjv469XQoUm1Qj6PFJQoVwXYmdgeXCqg5m")),
    ]

    borshInstructionSchema = borsh.struct([
        borsh.u8('variant'),
        borsh.str('name'),
        borsh.str('message'),
    ])

    static borshAccountSchema = borsh.struct([
        borsh.str("discriminator"),
        borsh.publicKey("reviewer"),
        borsh.bool('initialized'),
        borsh.str('name'),
        borsh.str('message'),
    ])

    serialize(instruction: number): Buffer {
        const buffer = Buffer.alloc(1000)
        this.borshInstructionSchema.encode(
            { ...this, variant: instruction }, buffer)
        return buffer.slice(0, this.borshInstructionSchema.getSpan(buffer))
    }

    static deserialize(buffer?: Buffer): StudentIntro|null {
        if (!buffer) {
            return null
        }

        try {
            const { name, message, reviewer } = 
            this.borshAccountSchema.decode(buffer)
            return new StudentIntro(name, message, reviewer)
        } catch(e) {
            console.log('Deserialization error:', e)
            console.log(buffer)
            return null
        }
    }
}