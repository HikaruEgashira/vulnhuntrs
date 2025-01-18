public class TestClass {
    private String field;

    public TestClass() {
        this.field = "test";
    }

    public void testMethod() {
        System.out.println(field);
    }

    interface TestInterface {
        void interfaceMethod();
    }

    enum TestEnum {
        ONE, TWO, THREE
    }
}
