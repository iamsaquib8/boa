//! Builtins live here, such as Object, String, Math, etc.

pub mod array;
pub mod bigint;
pub mod boolean;
pub mod console;
pub mod date;
pub mod error;
pub mod function;
pub mod global_this;
pub mod infinity;
pub mod json;
pub mod map;
pub mod math;
pub mod nan;
pub mod number;
pub mod object;
pub mod regexp;
pub mod string;
pub mod symbol;
pub mod undefined;

pub(crate) use self::{
    array::Array,
    bigint::BigInt,
    boolean::Boolean,
    console::Console,
    date::Date,
    error::{Error, RangeError, ReferenceError, SyntaxError, TypeError},
    function::BuiltInFunctionObject,
    global_this::GlobalThis,
    infinity::Infinity,
    json::Json,
    map::Map,
    math::Math,
    nan::NaN,
    number::Number,
    object::Object as BuiltInObjectObject,
    regexp::RegExp,
    string::String,
    symbol::Symbol,
    undefined::Undefined,
};
use crate::{
    builtins::function::{Function, FunctionFlags, NativeFunction},
    context::StandardConstructor,
    object::{GcObject, Object, ObjectData, PROTOTYPE},
    property::{Attribute, Property, PropertyKey},
    Context, Value,
};

#[derive(Debug)]
pub struct ObjectBuilder<'context> {
    context: &'context mut Context,
    object: GcObject,
}

impl<'context> ObjectBuilder<'context> {
    pub fn new(context: &'context mut Context) -> Self {
        let object = context.construct_object();
        Self { context, object }
    }

    pub fn static_method(
        &mut self,
        function: NativeFunction,
        name: &str,
        length: usize,
    ) -> &mut Self {
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .standard_objects()
                .function_object()
                .prototype()
                .into(),
        );
        function.insert_field("length", length.into());
        function.insert_field("name", name.into());

        self.object.borrow_mut().insert_field(name, function.into());
        self
    }

    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = Property::data_descriptor(value.into(), attribute);
        self.object.borrow_mut().insert_property(key, property);
        self
    }

    fn build(&mut self) -> Value {
        self.object.clone().into()
    }
}

use std::string::String as StdString;

#[allow(missing_debug_implementations)]
pub struct ConstructorBuilder<'context> {
    context: &'context mut Context,
    constrcutor_function: NativeFunction,
    constructor_object: GcObject,
    prototype: GcObject,
    name: StdString,
    length: usize,
    callable: bool,
    constructable: bool,
}

impl<'context> ConstructorBuilder<'context> {
    pub fn new(context: &'context mut Context, constructor: NativeFunction) -> Self {
        Self {
            context,
            constrcutor_function: constructor,
            constructor_object: GcObject::new(Object::default()),
            prototype: GcObject::new(Object::default()),
            length: 0,
            name: "[Object]".to_string(),
            callable: true,
            constructable: true,
        }
    }

    pub(crate) fn with_standard_object(
        context: &'context mut Context,
        constructor: NativeFunction,
        object: StandardConstructor,
    ) -> Self {
        Self {
            context,
            constrcutor_function: constructor,
            constructor_object: object.constructor,
            prototype: object.prototype,
            length: 0,
            name: "[Object]".to_string(),
            callable: true,
            constructable: true,
        }
    }

    pub fn method(&mut self, function: NativeFunction, name: &str, length: usize) -> &mut Self {
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .standard_objects()
                .function_object()
                .prototype()
                .into(),
        );
        function.insert_field("length", length.into());
        function.insert_field("name", name.into());

        self.prototype
            .borrow_mut()
            .insert_field(name, function.into());
        self
    }

    pub fn static_method(
        &mut self,
        function: NativeFunction,
        name: &str,
        length: usize,
    ) -> &mut Self {
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .standard_objects()
                .function_object()
                .prototype()
                .into(),
        );
        function.insert_field("length", length.into());
        function.insert_field("name", name.into());

        self.constructor_object
            .borrow_mut()
            .insert_field(name, function.into());
        self
    }

    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = Property::data_descriptor(value.into(), attribute);
        self.prototype.borrow_mut().insert_property(key, property);
        self
    }

    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = Property::data_descriptor(value.into(), attribute);
        self.constructor_object
            .borrow_mut()
            .insert_property(key, property);
        self
    }

    pub fn length(&mut self, length: usize) -> &mut Self {
        self.length = length;
        self
    }

    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: Into<StdString>,
    {
        self.name = name.into();
        self
    }

    pub fn callable(&mut self, callable: bool) -> &mut Self {
        self.callable = callable;
        self
    }

    pub fn constructable(&mut self, constructable: bool) -> &mut Self {
        self.constructable = constructable;
        self
    }

    fn build(&mut self) -> Value {
        // Create the native function
        let function = Function::BuiltIn(
            self.constrcutor_function.into(),
            FunctionFlags::from_parameters(self.callable, self.constructable),
        );

        let length = Property::data_descriptor(
            self.length.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        let mut name = StdString::new();
        std::mem::swap(&mut self.name, &mut name);
        let name = Property::data_descriptor(
            name.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        {
            let mut constructor = self.constructor_object.borrow_mut();
            constructor.data = ObjectData::Function(function);
            constructor.insert_property("length", length);
            constructor.insert_property("name", name);

            constructor.set_prototype_instance(
                self.context
                    .standard_objects()
                    .function_object()
                    .prototype()
                    .into(),
            );

            constructor.insert_field(PROTOTYPE, self.prototype.clone().into());
        }

        {
            let mut prototype = self.prototype.borrow_mut();
            prototype.insert_field("constructor", self.constructor_object.clone().into());
            prototype.set_prototype_instance(
                self.context
                    .standard_objects()
                    .object_object()
                    .prototype()
                    .into(),
            );
        }

        self.constructor_object.clone().into()
    }
}

pub trait BuiltIn {
    /// The binding name of the property.
    const NAME: &'static str;

    fn attribute() -> Attribute {
        Attribute::all()
    }
    fn init(context: &mut Context) -> (&'static str, Value, Attribute);
}

/// Initializes builtin objects and functions
#[inline]
pub fn init(context: &mut Context) {
    let globals2 = [
        // Global properties.
        Undefined::init,
        Infinity::init,
        NaN::init,
        GlobalThis::init,
        BuiltInFunctionObject::init,
        BuiltInObjectObject::init,
        Math::init,
        Json::init,
        Console::init,
        Array::init,
        BigInt::init,
        Boolean::init,
        Date::init,
        Map::init,
        Number::init,
        String::init,
        RegExp::init,
        Symbol::init,
        Error::init,
        RangeError::init,
        ReferenceError::init,
        TypeError::init,
        SyntaxError::init,
    ];

    let global_object = if let Value::Object(global) = context.global_object() {
        global.clone()
    } else {
        unreachable!("global object should always be an object")
    };

    for init in &globals2 {
        let (name, value, attribute) = init(context);
        let property = Property::data_descriptor(value, attribute);
        global_object.borrow_mut().insert_property(name, property);
    }
}
