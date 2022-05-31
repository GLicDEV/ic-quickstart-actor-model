import React from 'react'
import { ic_quickstart_actor_model } from "../../declarations/ic_quickstart_actor_model"

import { createActor } from "../../declarations/ic_quickstart_actor_model"

import { Secp256k1KeyIdentity } from "@dfinity/identity"

const Greet = () => {

    const handleSubmit = async (e) => {
        e.preventDefault();

        const greet = await ic_quickstart_actor_model.greet("test");
        console.log(greet)

        let canisterId = "rrkah-fqaaa-aaaaa-aaaaq-cai";


        const rawBuffer = new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        const test_id = Secp256k1KeyIdentity.generate(rawBuffer);

        const test = createActor(canisterId, {
            agentOptions: {
                identity: test_id,
            }
        });

        const greet2 = await test.greet("test2");
        console.log(greet2)

    }

    return (<>
        <div>Greet</div>
        <form onSubmit={handleSubmit}>
            <button id="clickMeBtn" type="submit">Click Me!</button>

        </form>
    </>
    )
}

export default Greet