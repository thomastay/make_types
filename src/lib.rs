#![allow(missing_docs)] // TODO
#![allow(dead_code)] // TODO
#![deny(rust_2018_idioms)]
#![deny(clippy::too_many_arguments)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![forbid(unsafe_code)]
#![warn(clippy::style)]
#![warn(clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_same_arms)]

use std::collections::HashMap;

#[derive(PartialEq, Eq, Clone)] // default deep equality
enum Shape {
    Bottom,
    Any(AnyShape),
    Bool,
    Int,
    Float,
    Rec(RecordShape),      // Records
    Coll(CollectionShape), // Collections
    Str,                   // Strings
    Null,
    NullBool,
    NullInt,
    NullFloat,
    NullRec(RecordShape),
    NullColl(CollectionShape), // Collections
    NullStr,
}

impl Shape {
    fn make_nullable(&self) -> Self {
        use Shape::*;
        match self {
            Bottom => panic!("Bottom is sub class of null"),
            Any(_) => self.clone(), // XXX: Expensive clone
            Bool => NullBool,
            Int => NullInt,
            Float => NullFloat,
            Rec(r) => NullRec(r.clone()),
            Coll(c) => NullColl(c.clone()),
            Str => NullStr,
            // returns the same
            Null | NullBool | NullInt | NullFloat | NullStr | NullRec(_) | NullColl(_) => {
                self.clone()
            }
        }
    }
    fn make_non_nullable(&self) -> Self {
        use Shape::*;
        match self {
            Bottom => panic!("Bottom is sub class of null"),
            Null => panic!("Null is sub class of non-nullable"),
            Any(_) => self.clone(), // XXX: Expensive clone
            NullBool => Bool,
            NullInt => Int,
            NullFloat => Float,
            NullRec(r) => Rec(r.clone()),
            NullColl(c) => Coll(c.clone()),
            NullStr => Str,
            // returns the same
            Bool | Int | Float | Str | Rec(_) | Coll(_) => self.clone(),
        }
    }
    fn is_nullable(&self) -> bool {
        use Shape::*;
        match self {
            Bottom | Any(_) | Bool | Int | Float | Rec(_) | Coll(_) | Str => false,
            Null | NullBool | NullInt | NullFloat | NullStr | NullRec(_) | NullColl(_) => true,
        }
    }
}

// enum Shape2 {
//     Bottom,
//     Any(AnyShape),
//     Bool(bool),
//     Int(bool),
//     Float(bool),
//     Rec(RecordShape),      // Records
//     Coll(CollectionShape), // Collections
//     Str,                   // Strings
// }

/// ----tag-----| --- data ------
///    1        |  `HashSet`

#[derive(PartialEq, Eq, Clone)] // default deep equality
struct AnyShape {
    /// The list of shapes that the Any Shape holds
    /// e.g. the type float | int --> shapes = [float, int]
    // shapes: HashSet<Shape>,
    shapes: Vec<Shape>,
}

// impl AnyShape {
//     fn make_nullable(&self) -> Self {
//         let shapes = self
//             .shapes
//             .iter()
//             .map(Shape::make_nullable)
//             .collect::<Vec<Shape>>();
//         AnyShape { shapes }
//     }
// }

// s1, s2 : AnyShape

#[derive(PartialEq, Eq, Clone)] // default deep equality
struct RecordShape {
    fields: HashMap<String, Shape>,
    // no idea
    // contexts: Vec<String>,
}

#[derive(Clone)] // default deep equality
struct CollectionShape {
    base: Box<Shape>, // Box<T> --> T is allocated on the heap
                      // no idea
                      // contexts: Vec<String>,
}

impl CollectionShape {
    fn vedant() -> i32 {
        5
    }
}

// PartialEq is a trait

// fn asd() {
//     let x = CollectionShape {
//         base: Box::new(Shape::Bottom), // box has a destructor
//         contexts: Vec::new(),
//     };
//     // x.vedant();
//     //x == x;
//     // CollectionShape::vedant(&x);
//     // int* base = malloc(sizeof(int));
//     // auto x = CollectionShape { .base=base, .contexts=[] }
//     // free(base);
//     5.fuck_off();
// }

trait FuckOff {
    fn fuck_off(&self) -> String;
}

impl FuckOff for i32 {
    #[must_use]
    fn fuck_off(&self) -> String {
        // format!("")
        format!("Fuck off {} times", self)
    }
}

// https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
impl PartialEq for CollectionShape {
    // &self is special --> keyword in Rust
    // x.asds()  x--> &self.
    fn eq(&self, other: &CollectionShape) -> bool {
        (*self.base) == *other.base // Box<T> -> T -> &T
    }
}

// https://doc.rust-lang.org/std/cmp/trait.Eq.html
// No implementation needed. When you do ==, it actually invokes PartialEq, not Eq.
impl Eq for CollectionShape {}

/// Creates a new Common Preferred Shape of s1 and s2, according to Fig 2 of the paper.
fn common_preferred_shape(s1: &Shape, s2: &Shape) -> Shape {
    fn new_any(a: &AnyShape, s: &Shape) -> Shape {
        let mut shape_clones = a.shapes.clone();
        shape_clones.push(s.clone());
        Any(AnyShape {
            shapes: shape_clones,
        })
    }

    use Shape::*;
    if s1 == s2 {
        return s1.clone();
    }
    match (s1, s2) {
        (Bottom, _) => s2.clone(),
        (_, Bottom) => s1.clone(),
        (Coll(c1), Coll(c2)) => Coll(CollectionShape {
            base: Box::new(common_preferred_shape(&c1.base, &c2.base)),
        }),
        (Null, _) => s2.make_nullable(),
        (_, Null) => s1.make_nullable(),
        (Any(a1), _) => new_any(a1, s2),
        (_, Any(a2)) => new_any(a2, s1),
        (Rec(r1), Rec(r2)) => {
            let mut fields: HashMap<String, Shape> = HashMap::new();
            for (r1_field, r1_field_shape) in &r1.fields {
                // Get the type of the field in r2, if it exists, else return the Null Shape
                // Then, extract the CSH of both, and insert it into the fields map
                let r2_field_shape = r2.fields.get(r1_field).unwrap_or(&Null);
                fields.insert(
                    r1_field.clone(),
                    common_preferred_shape(r1_field_shape, r2_field_shape),
                );
            }
            // TODO s2 Jul 17
            Rec(RecordShape { fields })
        }
        _ => {
            if s1.is_nullable() {
                common_preferred_shape(&s1.make_non_nullable(), s2).make_nullable()
            } else if s2.is_nullable() {
                common_preferred_shape(s1, &s2.make_non_nullable()).make_nullable()
            } else {
                // TODO s2 Jul 17
                todo!("Fill in Any");
            }
        }
    }
}

// int --> Integer
// 1   --> Box::new(1)

/*

{
    "a": 1,
    "b": 1,
    "c": 1
} // r1, nullable<r1>
{
    "d": 1,
    "e": 1,
    "f": 1
} // r2, nullable<r2>
*/

/*
export const enum BaseShape {
  BOTTOM,
  NULL,
  RECORD,
  STRING,
  BOOLEAN,
  NUMBER,
  COLLECTION,
  ANY,
}

export type Shape =
  | CBottomShape
  | CNullShape
  | CRecordShape
  | CStringShape
  | CBooleanShape
  | CNumberShape
  | CCollectionShape
  | CAnyShape;

export const enum ContextType {
  ENTITY,
  FIELD,
}

function pascalCase(n: string): string {
  return n
    .split("_")
    .map(s => (s[0] ? s[0].toUpperCase() : "") + s.slice(1))
    .join("");
}

export function getReferencedRecordShapes(
  e: Emitter,
  s: Set<CRecordShape>,
  sh: Shape,
): void {
  switch (sh.type) {
    case BaseShape.RECORD:
      if (!s.has(sh)) {
        s.add(sh);
        sh.getReferencedRecordShapes(e, s);
      }
      break;
    case BaseShape.COLLECTION:
      getReferencedRecordShapes(e, s, sh.baseShape);
      break;
    case BaseShape.ANY:
      sh.getDistilledShapes(e).forEach(sh =>
        getReferencedRecordShapes(e, s, sh),
      );
      break;
  }
}

export class FieldContext {
  public get type(): ContextType.FIELD {
    return ContextType.FIELD;
  }
  public readonly parent: CRecordShape;
  public readonly field: string;
  constructor(parent: CRecordShape, field: string) {
    this.parent = parent;
    this.field = field;
  }
  public getName(_e: Emitter): string {
    const name = pascalCase(this.field);
    return name;
  }
}

export class EntityContext {
  public get type(): ContextType.ENTITY {
    return ContextType.ENTITY;
  }
  public readonly parent: CCollectionShape;
  constructor(parent: CCollectionShape) {
    this.parent = parent;
  }
  public getName(e: Emitter): string {
    return `${this.parent.getName(e)}Entity`;
  }
}

export type Context = FieldContext | EntityContext;

export class CBottomShape {
  public get type(): BaseShape.BOTTOM {
    return BaseShape.BOTTOM;
  }
  public get nullable(): boolean {
    return false;
  }
  public makeNullable(): CBottomShape {
    throw new TypeError(`Doesn't make sense.`);
  }
  public makeNonNullable(): CBottomShape {
    return this;
  }
  public emitType(_e: Emitter): void {
    throw new Error(`Doesn't make sense.`);
  }
  public getProxyType(_e: Emitter): string {
    throw new Error(`Doesn't make sense.`);
  }
  public equal(t: Shape): boolean {
    return this === t;
  }
}

export const BottomShape = new CBottomShape();

export class CNullShape {
  public get nullable(): boolean {
    return true;
  }
  public get type(): BaseShape.NULL {
    return BaseShape.NULL;
  }
  public makeNullable(): CNullShape {
    return this;
  }
  public makeNonNullable(): CNullShape {
    return this;
  }
  public emitType(e: Emitter): void {
    e.interfaces.write("null");
  }
  public getProxyType(_e: Emitter): string {
    return "null";
  }
  public equal(t: Shape): boolean {
    return this === t;
  }
}

export const NullShape = new CNullShape();

export class CNumberShape {
  public get nullable(): boolean {
    return this === NullableNumberShape;
  }
  public get type(): BaseShape.NUMBER {
    return BaseShape.NUMBER;
  }
  public makeNullable(): CNumberShape {
    return NullableNumberShape;
  }
  public makeNonNullable(): CNumberShape {
    return NumberShape;
  }
  public emitType(e: Emitter): void {
    e.interfaces.write(this.getProxyType(e));
  }
  public getProxyType(_e: Emitter): string {
    let rv = "number";
    if (this.nullable) {
      rv += " | null";
    }
    return rv;
  }
  public equal(t: Shape): boolean {
    return this === t;
  }
}

export const NumberShape = new CNumberShape();
export const NullableNumberShape = new CNumberShape();

export class CStringShape {
  public get type(): BaseShape.STRING {
    return BaseShape.STRING;
  }
  public get nullable(): boolean {
    return this === NullableStringShape;
  }
  public makeNullable(): CStringShape {
    return NullableStringShape;
  }
  public makeNonNullable(): CStringShape {
    return StringShape;
  }
  public emitType(e: Emitter): void {
    e.interfaces.write(this.getProxyType(e));
  }
  public getProxyType(_e: Emitter): string {
    let rv = "string";
    if (this.nullable) {
      rv += " | null";
    }
    return rv;
  }
  public equal(t: Shape): boolean {
    return this === t;
  }
}

export const StringShape = new CStringShape();
export const NullableStringShape = new CStringShape();

export class CBooleanShape {
  public get type(): BaseShape.BOOLEAN {
    return BaseShape.BOOLEAN;
  }
  public get nullable(): boolean {
    return this === NullableBooleanShape;
  }
  public makeNullable(): CBooleanShape {
    return NullableBooleanShape;
  }
  public makeNonNullable(): CBooleanShape {
    return BooleanShape;
  }
  public emitType(e: Emitter): void {
    e.interfaces.write(this.getProxyType(e));
  }
  public getProxyType(_e: Emitter): string {
    let rv = "boolean";
    if (this.nullable) {
      rv += " | null";
    }
    return rv;
  }
  public equal(t: Shape): boolean {
    return this === t;
  }
}

export const BooleanShape = new CBooleanShape();
export const NullableBooleanShape = new CBooleanShape();

export class CAnyShape {
  public get type(): BaseShape.ANY {
    return BaseShape.ANY;
  }
  private readonly _shapes: Shape[];
  private readonly _nullable: boolean = false;
  private _hasDistilledShapes: boolean = false;
  private _distilledShapes: Shape[] = [];
  constructor(shapes: Shape[], nullable: boolean) {
    this._shapes = shapes;
    this._nullable = nullable;
  }
  public get nullable(): boolean {
    return this._nullable === true;
  }
  public makeNullable(): CAnyShape {
    if (this._nullable) {
      return this;
    } else {
      return new CAnyShape(this._shapes, true);
    }
  }
  public makeNonNullable(): CAnyShape {
    if (this._nullable) {
      return new CAnyShape(this._shapes, false);
    } else {
      return this;
    }
  }
  private _ensureDistilled(e: Emitter): void {
    if (!this._hasDistilledShapes) {
      let shapes = new Map<BaseShape, Shape[]>();
      for (let i = 0; i < this._shapes.length; i++) {
        const s = this._shapes[i];
        if (!shapes.has(s.type)) {
          shapes.set(s.type, []);
        }
        shapes.get(s.type)!.push(s);
      }
      shapes.forEach((shapes, _key) => {
        let shape: Shape = BottomShape;
        for (let i = 0; i < shapes.length; i++) {
          shape = csh(e, shape, shapes[i]);
        }
        this._distilledShapes.push(shape);
      });
      this._hasDistilledShapes = true;
    }
  }
  public getDistilledShapes(e: Emitter): Shape[] {
    this._ensureDistilled(e);
    return this._distilledShapes;
  }
  public addToShapes(shape: Shape): CAnyShape {
    const shapeClone = this._shapes.slice(0);
    shapeClone.push(shape);
    return new CAnyShape(shapeClone, this._nullable);
  }
  public emitType(e: Emitter): void {
    this._ensureDistilled(e);
    this._distilledShapes.forEach((s, i) => {
      s.emitType(e);
      if (i < this._distilledShapes.length - 1) {
        e.interfaces.write(" | ");
      }
    });
  }
  public getProxyType(e: Emitter): string {
    this._ensureDistilled(e);
    return this._distilledShapes.map(s => s.getProxyType(e)).join(" | ");
  }
  public equal(t: Shape): boolean {
    return this === t;
  }
}

export class CRecordShape {
  public get type(): BaseShape.RECORD {
    return BaseShape.RECORD;
  }
  private readonly _nullable: boolean;
  private readonly _fields: Map<string, Shape>;
  public readonly contexts: Context[];

  private _name: string | null = null;
  private constructor(
    fields: Map<string, Shape>,
    nullable: boolean,
    contexts: Context[],
  ) {
    // Assign a context to all fields.
    const fieldsWithContext = new Map<string, Shape>();
    fields.forEach((val, index) => {
      if (val.type === BaseShape.RECORD || val.type === BaseShape.COLLECTION) {
        fieldsWithContext.set(
          index,
          val.addContext(new FieldContext(this, index)),
        );
      } else {
        fieldsWithContext.set(index, val);
      }
    });
    this._fields = fieldsWithContext;
    this._nullable = nullable;
    this.contexts = contexts;
  }
  public get nullable(): boolean {
    return this._nullable;
  }
  /**
   * Construct a new record shape. Returns an existing, equivalent record shape
   * if applicable.
   */
  public static Create(
    e: Emitter,
    fields: Map<string, Shape>,
    nullable: boolean,
    contexts: Context[] = [],
  ): CRecordShape {
    const record = new CRecordShape(fields, nullable, contexts);
    return e.registerRecordShape(record);
  }
  public makeNullable(): CRecordShape {
    if (this._nullable) {
      return this;
    } else {
      return new CRecordShape(this._fields, true, this.contexts);
    }
  }
  public addContext(ctx: Context): CRecordShape {
    this.contexts.push(ctx);
    return this;
  }
  public makeNonNullable(): CRecordShape {
    if (this._nullable) {
      return new CRecordShape(this._fields, false, this.contexts);
    } else {
      return this;
    }
  }
  public forEachField(cb: (t: Shape, name: string) => any): void {
    this._fields.forEach(cb);
  }
  public getField(name: string): Shape {
    const t = this._fields.get(name);
    if (!t) {
      return NullShape;
    } else {
      return t;
    }
  }
  public equal(t: Shape): boolean {
    if (
      t.type === BaseShape.RECORD &&
      this._nullable === t._nullable &&
      this._fields.size === t._fields.size
    ) {
      let rv = true;
      const tFields = t._fields;
      // Check all fields.
      // NOTE: Since size is equal, no need to iterate over t. Either they have the same fields
      // or t is missing fields from this one.
      this.forEachField((t, name) => {
        if (rv) {
          const field = tFields.get(name);
          if (field) {
            rv = field.equal(t);
          } else {
            rv = false;
          }
        }
      });
      return rv;
    }
    return false;
  }
  public emitType(e: Emitter): void {
    e.interfaces.write(this.getName(e));
    if (this.nullable) {
      e.interfaces.write(" | null");
    }
  }
  public getProxyClass(e: Emitter): string {
    return `${this.getName(e)}Proxy`;
  }
  public getProxyType(e: Emitter): string {
    let rv = `${this.getName(e)}Proxy`;
    if (this.nullable) {
      rv += " | null";
    }
    return rv;
  }
  public emitInterfaceDefinition(e: Emitter): void {
    const w = e.interfaces;
    w.write(`export interface ${this.getName(e)} {`).endl();
    this.forEachField((t, name) => {
      w.tab(1).write(name);
      if (t.nullable) {
        w.write("?");
      }
      w.write(": ");
      t.emitType(e);
      w.write(";").endl();
    });
    w.write(`}`);
  }
  public emitProxyClass(e: Emitter): void {
    const w = e.proxies;
    w.writeln(`export class ${this.getProxyClass(e)} {`);
    this.forEachField((t, name) => {
      w.tab(1).writeln(`public readonly ${name}: ${t.getProxyType(e)};`);
    });
    w.tab(1).writeln(
      `public static Parse(d: string): ${this.getProxyType(e)} {`,
    );
    w.tab(2).writeln(`return ${this.getProxyClass(e)}.Create(JSON.parse(d));`);
    w.tab(1).writeln(`}`);
    w.tab(1).writeln(
      `public static Create(d: any, field: string = 'root'): ${this.getProxyType(
        e,
      )} {`,
    );
    w.tab(2).writeln(`if (!field) {`);
    w.tab(3).writeln(`obj = d;`);
    w.tab(3).writeln(`field = "root";`);
    w.tab(2).writeln(`}`);
    w.tab(2).writeln(`if (d === null || d === undefined) {`);
    w.tab(3);
    if (this.nullable) {
      w.writeln(`return null;`);
    } else {
      e.markHelperAsUsed("throwNull2NonNull");
      w.writeln(`throwNull2NonNull(field, d);`);
    }
    w.tab(2).writeln(`} else if (typeof(d) !== 'object') {`);
    e.markHelperAsUsed("throwNotObject");
    w.tab(3).writeln(`throwNotObject(field, d, ${this.nullable});`);
    w.tab(2).writeln(`} else if (Array.isArray(d)) {`);
    e.markHelperAsUsed("throwIsArray");
    w.tab(3).writeln(`throwIsArray(field, d, ${this.nullable});`);
    w.tab(2).writeln(`}`);
    // At this point, we know we have a non-null object.
    // Check all fields.
    this.forEachField((t, name) => {
      emitProxyTypeCheck(e, w, t, 2, `d.${name}`, `field + ".${name}"`);
    });
    w.tab(2).writeln(`return new ${this.getProxyClass(e)}(d);`);
    w.tab(1).writeln(`}`);
    w.tab(1).writeln(`private constructor(d: any) {`);
    // Emit an assignment for each field.
    this.forEachField((_t, name) => {
      w.tab(2).writeln(`this.${name} = d.${name};`);
    });
    w.tab(1).writeln(`}`);
    w.writeln("}");
  }
  public getReferencedRecordShapes(e: Emitter, rv: Set<CRecordShape>): void {
    this.forEachField((t, _name) => {
      getReferencedRecordShapes(e, rv, t);
    });
  }
  public markAsRoot(name: string): void {
    this._name = name;
  }
  public getName(e: Emitter): string {
    if (typeof this._name === "string") {
      return this._name;
    }
    // Calculate unique name.
    const nameSet = new Set<string>();
    let name = this.contexts
      .map(c => c.getName(e))
      // Remove duplicate names.
      .filter(n => {
        if (!nameSet.has(n)) {
          nameSet.add(n);
          return true;
        }
        return false;
      })
      .join("Or");
    this._name = e.registerName(name);
    return this._name;
  }
}

export class CCollectionShape {
  public get type(): BaseShape.COLLECTION {
    return BaseShape.COLLECTION;
  }
  public readonly baseShape: Shape;
  public readonly contexts: Context[];
  private _name: string | null = null;
  constructor(baseShape: Shape, contexts: Context[] = []) {
    // Add context if a record/collection.
    this.baseShape =
      baseShape.type === BaseShape.RECORD ||
      baseShape.type === BaseShape.COLLECTION
        ? baseShape.addContext(new EntityContext(this))
        : baseShape;
    this.contexts = contexts;
  }

  public get nullable(): boolean {
    return true;
  }
  public makeNullable(): CCollectionShape {
    return this;
  }
  public makeNonNullable(): CCollectionShape {
    return this;
  }
  public addContext(ctx: Context): CCollectionShape {
    this.contexts.push(ctx);
    return this;
  }
  public emitType(e: Emitter): void {
    e.interfaces.write("(");
    this.baseShape.emitType(e);
    e.interfaces.write(")[] | null");
  }
  public getProxyType(e: Emitter): string {
    const base = this.baseShape.getProxyType(e);
    if (base.indexOf("|") !== -1) {
      return `(${base})[] | null`;
    } else {
      return `${base}[] | null`;
    }
  }
  public equal(t: Shape): boolean {
    return t.type === BaseShape.COLLECTION && this.baseShape.equal(t.baseShape);
  }
  public getName(e: Emitter): string {
    if (typeof this._name === "string") {
      return this._name;
    }
    const nameSet = new Set<string>();
    // No need to make collection names unique.
    this._name = this.contexts
      .map(c => c.getName(e))
      .filter(name => {
        if (!nameSet.has(name)) {
          nameSet.add(name);
          return true;
        }
        return false;
      })
      .join("Or");
    return this._name;
  }
}

export function csh(e: Emitter, s1: Shape, s2: Shape): Shape {
  // csh(σ, σ) = σ
  if (s1 === s2) {
    return s1;
  }
  if (s1.type === BaseShape.COLLECTION && s2.type === BaseShape.COLLECTION) {
    // csh([σ1], [σ2]) = [csh(σ1, σ2)]
    return new CCollectionShape(csh(e, s1.baseShape, s2.baseShape));
  }
  // csh(⊥, σ) = csh(σ, ⊥) = σ
  if (s1.type === BaseShape.BOTTOM) {
    return s2;
  }
  if (s2.type === BaseShape.BOTTOM) {
    return s1;
  }

  // csh(null, σ) = csh(σ, null) = nullable<σ>
  if (s1.type === BaseShape.NULL) {
    return s2.makeNullable();
  }
  if (s2.type === BaseShape.NULL) {
    return s1.makeNullable();
  }

  // csh(any, σ) = csh(σ, any) = any
  if (s1.type === BaseShape.ANY) {
    return s1.addToShapes(s2);
  }
  if (s2.type === BaseShape.ANY) {
    return s2.addToShapes(s1);
  }

  // csh(σ2, nullable<σˆ1> ) = csh(nullable<σˆ1> , σ2) = nullable<csh(σˆ1, σ2)>
  if (s1.nullable && s1.type !== BaseShape.COLLECTION) {
    return csh(e, s1.makeNonNullable(), s2).makeNullable();
  }
  if (s2.nullable && s2.type !== BaseShape.COLLECTION) {
    return csh(e, s2.makeNonNullable(), s1).makeNullable();
  }

  // (recd) rule
  if (s1.type === BaseShape.RECORD && s2.type === BaseShape.RECORD) {
    // Get all fields.
    const fields = new Map<string, Shape>();
    s1.forEachField((t, name) => {
      fields.set(name, csh(e, t, s2.getField(name)));
    });
    s2.forEachField((t, name) => {
      if (!fields.has(name)) {
        fields.set(name, csh(e, t, s1.getField(name)));
      }
    });
    return CRecordShape.Create(e, fields, false);
  }

  // (any) rule
  return new CAnyShape([s1, s2], s1.nullable || s2.nullable);
}

export function d2s(e: Emitter, d: any): Shape {
  if (d === undefined || d === null) {
    return NullShape;
  }
  switch (typeof d) {
    case "number":
      return NumberShape;
    case "string":
      return StringShape;
    case "boolean":
      return BooleanShape;
  }

  // Must be an object or array.
  if (Array.isArray(d)) {
    // Empty array: Not enough information to figure out a precise type.
    if (d.length === 0) {
      return new CCollectionShape(NullShape);
    }
    let t: Shape = BottomShape;
    for (let i = 0; i < d.length; i++) {
      t = csh(e, t, d2s(e, d[i]));
    }
    return new CCollectionShape(t);
  }

  const keys = Object.keys(d);
  const fields = new Map<string, Shape>();
  for (let i = 0; i < keys.length; i++) {
    const name = keys[i];
    fields.set(name, d2s(e, d[name]));
  }
  return CRecordShape.Create(e, fields, false);
}

 */
