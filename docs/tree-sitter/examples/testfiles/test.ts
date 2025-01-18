interface TestInterface {
    name: string;
    value: number;
}

class TestTypeScriptClass implements TestInterface {
    name: string;
    value: number;

    constructor(name: string, value: number) {
        this.name = name;
        this.value = value;
    }

    public getValue(): number {
        return this.value;
    }
}

function testTypeScriptFunction(param: TestInterface): string {
    return param.name;
}

const typeScriptConst: number = 42;
let typeScriptLet: string = "test";

type CustomType = {
    id: number;
    label: string;
};
