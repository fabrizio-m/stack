// trait Instruction {
// const POP: usize;
// const PUSH: usize;
// }

use std::{marker::PhantomData, ops};

struct EmptyStack;
struct StackOfOne<O>(O);
struct StackOfTwo<O>(O, O);
// enum StackOfN<O> {
// Three(StackOfTwo<O>, O),
// MoreThanThree(Box<Self>, O),
// }

trait Popable<O> {
    type NewStack;
    fn pop(self) -> (Self::NewStack, O);
}
trait Pushable<O> {
    type NewStack;
    fn push(self, h: O) -> Self::NewStack;
}
impl<O> Pushable<O> for EmptyStack {
    type NewStack = StackOfOne<O>;

    fn push(self, h: O) -> Self::NewStack {
        StackOfOne(h)
    }
}
impl<O> Popable<O> for StackOfOne<O> {
    type NewStack = EmptyStack;
    fn pop(self) -> (Self::NewStack, O) {
        (EmptyStack, self.0)
    }
}
impl<O> Pushable<O> for StackOfOne<O> {
    type NewStack = StackOfTwo<O>;

    fn push(self, h: O) -> Self::NewStack {
        let Self(x) = self;
        StackOfTwo(x, h)
    }
}
impl<O> Popable<O> for StackOfTwo<O> {
    type NewStack = StackOfOne<O>;
    fn pop(self) -> (Self::NewStack, O) {
        let StackOfTwo(a, b) = self;
        (StackOfOne(a), b)
    }
}
impl<O> Pushable<O> for StackOfTwo<O> {
    type NewStack = ArbitraryStack<O, Self>;

    fn push(self, h: O) -> Self::NewStack {
        ArbitraryStack {
            substack: self,
            head: h,
        }
    }
}

trait StackOfN {}
struct ArbitraryStack<O, S: StackOfN> {
    substack: S,
    head: O,
}
impl<O> StackOfN for StackOfTwo<O> {}
impl<O, S: StackOfN> StackOfN for ArbitraryStack<O, S> {}

impl<O, S: StackOfN> Popable<O> for ArbitraryStack<O, S> {
    type NewStack = S;

    fn pop(self) -> (Self::NewStack, O) {
        let ArbitraryStack { substack, head } = self;
        (substack, head)
    }
}
impl<O, S: StackOfN> Pushable<O> for ArbitraryStack<O, S> {
    type NewStack = ArbitraryStack<O, Self>;

    fn push(self, h: O) -> Self::NewStack {
        ArbitraryStack {
            substack: self,
            head: h,
        }
    }
}

trait Runner<O> {
    type Stack;
    type NewStack;
    type Pop;
    type Push;
    fn run<I: Instruction<O, Runner = Self>>(instruction: I, stack: Self::Stack) -> Self::NewStack;
}
trait Instruction<O> {
    type Runner: Runner<O>;
    fn operate(self, popped: <Self::Runner as Runner<O>>::Pop)
        -> <Self::Runner as Runner<O>>::Push;
}

struct PopRunner<O, S: Popable<O>>(PhantomData<(O, S)>);

impl<O, S> Runner<O> for PopRunner<O, S>
where
    S: Popable<O>,
{
    type Stack = S;

    type NewStack = S::NewStack;

    type Pop = O;

    type Push = ();

    fn run<I: Instruction<O, Runner = Self>>(instruction: I, stack: Self::Stack) -> Self::NewStack {
        let (new_stack, popped) = stack.pop();
        let _push = instruction.operate(popped);
        new_stack
    }
}
struct PushRunner<O, S>(PhantomData<(O, S)>);

impl<O, S> Runner<O> for PushRunner<O, S>
where
    S: Pushable<O>,
{
    type Stack = S;

    type NewStack = S::NewStack;

    type Pop = ();

    type Push = O;

    fn run<I: Instruction<O, Runner = Self>>(instruction: I, stack: Self::Stack) -> Self::NewStack {
        let push = instruction.operate(());
        stack.push(push)
    }
}
struct PopPushRunner<O, S1: Popable<O>, S2>(PhantomData<(O, S1, S2)>);

impl<O, S1, S2> Runner<O> for PopPushRunner<O, S1, S2>
where
    S1: Popable<O, NewStack = S2>,
    S2: Pushable<O>,
{
    type Stack = S1;

    type NewStack = S2::NewStack;

    type Pop = O;

    type Push = O;

    fn run<I: Instruction<O, Runner = Self>>(instruction: I, stack: Self::Stack) -> Self::NewStack {
        let (new_stack, popped) = stack.pop();
        let push = instruction.operate(popped);
        let new_stack = new_stack.push(push);
        new_stack
    }
}

struct PopPopPushRunner<O, S1, S2, S3>(PhantomData<(O, S1, S2, S3)>);

impl<O, S1, S2, S3> Runner<O> for PopPopPushRunner<O, S1, S2, S3>
where
    S1: Popable<O, NewStack = S2>,
    S2: Popable<O, NewStack = S3>,
    S3: Pushable<O>,
{
    type Stack = S1;

    type NewStack = S3::NewStack;

    type Pop = (O, O);

    type Push = O;

    fn run<I: Instruction<O, Runner = Self>>(instruction: I, stack: Self::Stack) -> Self::NewStack {
        let (stack, p1) = stack.pop();
        let (stack, p2) = stack.pop();
        let push = instruction.operate((p1, p2));
        let new_stack = stack.push(push);
        new_stack
    }
}

struct Push<O, S>(O, PhantomData<S>);
struct Pop<S>(PhantomData<S>);
struct Add<S1, S2, S3>(PhantomData<(S1, S2, S3)>);
struct Mul<S1, S2, S3>(PhantomData<(S1, S2, S3)>);
impl<O, S: Popable<O>> Instruction<O> for Pop<S> {
    type Runner = PopRunner<O, S>;

    fn operate(self, _popped: O) -> () {
        ()
    }
}
impl<O, S> Instruction<O> for Push<O, S>
where
    S: Pushable<O>,
{
    type Runner = PushRunner<O, S>;

    fn operate(
        self,
        _popped: <Self::Runner as Runner<O>>::Pop,
    ) -> <Self::Runner as Runner<O>>::Push {
        self.0
    }
}
impl<O, S1, S2, S3> Instruction<O> for Add<S1, S2, S3>
where
    //S: Pushable<O>,
    S1: Popable<O, NewStack = S2>,
    S2: Popable<O, NewStack = S3>,
    S3: Pushable<O>,
    O: ops::Add<Output = O>,
{
    type Runner = PopPopPushRunner<O, S1, S2, S3>;
    fn operate(
        self,
        popped: <Self::Runner as Runner<O>>::Pop,
    ) -> <Self::Runner as Runner<O>>::Push {
        let (a, b) = popped;
        let c = a + b;
        c
    }
}
impl<O, S1, S2, S3> Instruction<O> for Mul<S1, S2, S3>
where
    //S: Pushable<O>,
    S1: Popable<O, NewStack = S2>,
    S2: Popable<O, NewStack = S3>,
    S3: Pushable<O>,
    O: ops::Mul<Output = O>,
{
    type Runner = PopPopPushRunner<O, S1, S2, S3>;
    fn operate(
        self,
        popped: <Self::Runner as Runner<O>>::Pop,
    ) -> <Self::Runner as Runner<O>>::Push {
        let (a, b) = popped;
        let c = a * b;
        c
    }
}

trait PushInstruction<O> {
    fn new_val(self) -> O;
}
trait PopPushInstruction<O> {
    fn new_val(self, val: O) -> O;
}

fn intrerpret<O, S1, S2, R, I>(i: I, stack: S1) -> S2
where
    R: Runner<O, Stack = S1, NewStack = S2>,
    I: Instruction<O, Runner = R>,
{
    R::run(i, stack)
}

fn test() {
    let stack = EmptyStack;
    let stack = intrerpret(Push(8, PhantomData), stack);
    let stack = intrerpret(Pop, stack);
    let stack = intrerpret(Push(9, PhantomData), stack);
    let stack = intrerpret(Add(PhantomData), stack);
}
